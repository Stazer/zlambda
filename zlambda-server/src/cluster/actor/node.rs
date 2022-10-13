use crate::algorithm::next_key;
use crate::cluster::actor::PacketReaderActor;
use crate::cluster::NodeId;
use crate::cluster::{
    ConnectionId, LeaderConnectActorMessage, NodeActorRegisterMessage,
    NodeActorRemoveConnectionMessage, NodeRegisterResponsePacketError,
    NodeRegisterResponsePacketSuccessData, NodeRegisterResponsePacketSuccessNodeData, Packet,
    PacketReaderActorReadPacketMessage, ReadPacketError,
};
use crate::common::{
    ActorExecuteMessage, ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage,
    TcpStreamActor, TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, Recipient,
    ResponseActFuture, SpawnHandle, WrapFuture,
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::io;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum FollowerConnectionType {
    Leader { node_id: NodeId },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct FollowerConnection {
    connection_id: ConnectionId,
    r#type: FollowerConnectionType,
    stream_actor_address: Addr<TcpStreamActor>,
    reader_actor_address: Addr<PacketReaderActor<ConnectionId>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct FollowerNode {
    node_id: NodeId,
    socket_address: SocketAddr,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum LeaderConnectionType {
    Node { node_id: NodeId },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderConnection {
    connection_id: ConnectionId,
    r#type: Option<LeaderConnectionType>,
    stream_actor_address: Addr<TcpStreamActor>,
    reader_actor_address: Addr<PacketReaderActor<ConnectionId>>,
    peer_address: SocketAddr,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum LeaderNodeState {
    Registering,
    Registered {},
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderNode {
    node_id: NodeId,
    connection_id: ConnectionId,
    socket_address: SocketAddr,
    state: LeaderNodeState,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum FollowerState {
    Unregistered,
    Registering {
        stream_actor_address: Addr<TcpStreamActor>,
        reader_actor_address: Addr<PacketReaderActor<ConnectionId>>,
    },
    Registered {
        node_id: NodeId,
        leader_node_id: NodeId,
        leader_connection_id: NodeId,
        nodes: HashMap<NodeId, FollowerNode>,
        connections: HashMap<ConnectionId, FollowerConnection>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum Type {
    Leader {
        id: NodeId,
        listener_actor_address: Addr<TcpListenerActor>,
        listener_local_address: SocketAddr,
        connections: HashMap<ConnectionId, LeaderConnection>,
        nodes: HashMap<NodeId, LeaderNode>,
        heartbeat_spawn_handle: Option<SpawnHandle>,
    },
    Follower {
        state: FollowerState,
        listener_actor_address: Addr<TcpListenerActor>,
        listener_local_address: SocketAddr,
    },
    Candidate {},
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActor {
    r#type: Type,
}

impl Actor for NodeActor {
    type Context = Context<Self>;

    fn stopped(&mut self, context: &mut Self::Context) {
        match &self.r#type {
            Type::Leader { connections, listener_actor_address, mut heartbeat_spawn_handle, .. } => {
                listener_actor_address.do_send(ActorStopMessage);

                match heartbeat_spawn_handle.take() {
                    Some(heartbeat_spawn_handle) => {
                        context.cancel_future(heartbeat_spawn_handle);
                    },
                    None => {},
                };

                for connection in connections.values() {
                    connection.stream_actor_address.do_send(ActorStopMessage);
                    connection.reader_actor_address.do_send(ActorStopMessage);
                }
            }
            Type::Follower { state, listener_actor_address, .. } => {
                listener_actor_address.do_send(ActorStopMessage);

                match state {
                    FollowerState::Unregistered => {}
                    FollowerState::Registering {
                        stream_actor_address,
                        reader_actor_address,
                    } => {
                        stream_actor_address.do_send(ActorStopMessage);
                        reader_actor_address.do_send(ActorStopMessage);
                    }
                    FollowerState::Registered { connections, .. } => {
                        for connection in connections.values() {
                            connection.stream_actor_address.do_send(ActorStopMessage);
                            connection.reader_actor_address.do_send(ActorStopMessage);
                        }
                    }
                }
            },
            Type::Candidate {} => {}
        }

        trace!("Stopped");
    }
}

impl Handler<ActorStopMessage> for NodeActor {
    type Result = <ActorStopMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(&mut self, _message: ActorStopMessage, context: &mut <Self as Actor>::Context) {
        context.stop();
    }
}

impl Handler<PacketReaderActorReadPacketMessage<ConnectionId>> for NodeActor {
    type Result = <NodeActorRemoveConnectionMessage<ConnectionId> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage<ConnectionId>,
        context: &mut <Self as Actor>::Context,
    ) {
        let (connection_id, packet) = message.into();

        match &mut self.r#type {
            Type::Leader {
                id,
                ref mut connections,
                ref mut nodes,
                ..
            } => match packet {
                Packet::NodeRegisterRequest { local_address } => {
                    let mut connection = connections
                        .get_mut(&connection_id)
                        .expect("Invalid connection");

                    let node_id = next_key(&nodes);

                    nodes.insert(
                        node_id,
                        LeaderNode {
                            node_id,
                            connection_id,
                            socket_address: SocketAddr::new(
                                connection.peer_address.ip(),
                                local_address.port(),
                            ),
                            state: LeaderNodeState::Registering,
                        },
                    );

                    connection.r#type = Some(LeaderConnectionType::Node { node_id });

                    connection
                        .stream_actor_address
                        .try_send(TcpStreamActorSendMessage::new(
                            Packet::NodeRegisterResponse {
                                result: Ok(NodeRegisterResponsePacketSuccessData::new(
                                    node_id,
                                    nodes
                                        .values()
                                        .map(|node| {
                                            NodeRegisterResponsePacketSuccessNodeData::new(
                                                node.node_id,
                                                node.socket_address.clone(),
                                            )
                                        })
                                        .collect(),
                                    *id,
                                )),
                            }
                            .to_bytes()
                            .expect("Cannot write NodeRegisterReponse"),
                        ))
                        .expect("Cannot send NodeRegisterReponse");

                    trace!(
                        "Node {} registered with connection {}",
                        node_id,
                        connection_id
                    );
                }
                Packet::NodeRegisterAcknowledgement => {
                    let mut connection = connections
                        .get_mut(&connection_id)
                        .expect("Invalid connection");

                    let node_id = match connection.r#type {
                        Some(LeaderConnectionType::Node { node_id }) => node_id,
                        None => panic!("Invalid connection type"),
                    };

                    let mut node = nodes.get_mut(&node_id).expect("Invalid node");
                    node.state = LeaderNodeState::Registered {};
                }
                Packet::Pong => {
                    trace!("Received pong from connection {}", connection_id);
                }
                packet => {
                    error!(
                        "Received invalid packet {:?} from connection {} while being a leader",
                        packet, connection_id,
                    );
                }
            },
            Type::Follower { ref mut state, .. } => match state {
                FollowerState::Unregistered => match packet {
                    packet => {
                        error!(
                            "Received invalid packet {:?} from connection {} while being an unregistered follower",
                            packet,
                            connection_id,
                        );
                    }
                },
                FollowerState::Registering {
                    stream_actor_address,
                    reader_actor_address,
                } => match packet {
                    Packet::NodeRegisterResponse { result } => match result {
                        Ok(data) => {
                            let (node_id, nodes, leader_node_id): (
                                NodeId,
                                Vec<NodeRegisterResponsePacketSuccessNodeData>,
                                NodeId,
                            ) = data.into();

                            let stream_actor_address = stream_actor_address.clone();

                            *state = FollowerState::Registered {
                                node_id,
                                leader_node_id,
                                leader_connection_id: connection_id,
                                nodes: nodes
                                    .into_iter()
                                    .map(|node| {
                                        let (node_id, socket_address) = node.into();

                                        (
                                            node_id,
                                            FollowerNode {
                                                node_id,
                                                socket_address,
                                            },
                                        )
                                    })
                                    .collect(),
                                connections: HashMap::from([(
                                    connection_id,
                                    FollowerConnection {
                                        connection_id,
                                        r#type: FollowerConnectionType::Leader { node_id },
                                        stream_actor_address: stream_actor_address.clone(),
                                        reader_actor_address: reader_actor_address.clone(),
                                    },
                                )]),
                            };

                            stream_actor_address
                                .try_send(TcpStreamActorSendMessage::new(
                                    Packet::NodeRegisterAcknowledgement
                                        .to_bytes()
                                        .expect("Cannot write NodeRegisterAcknowledgement"),
                                ))
                                .expect("Cannot send NodeRegisterAcknowledgement");

                            trace!("Registered as node {}", node_id);
                        }
                        Err(NodeRegisterResponsePacketError::NotALeader { leader_address }) => {
                            stream_actor_address
                                .try_send(ActorStopMessage)
                                .expect("Cannot send ActorStopMessage");
                            reader_actor_address
                                .try_send(ActorStopMessage)
                                .expect("Cannot send ActorStopMessage");

                            *state = FollowerState::Unregistered;

                            error!("Not a leader");
                        }
                    },
                    packet => {
                        stream_actor_address
                            .try_send(ActorStopMessage)
                            .expect("Cannot send ActorStopMessage");
                        reader_actor_address
                            .try_send(ActorStopMessage)
                            .expect("Cannot send ActorStopMessage");

                        error!(
                            "Received invalid packet {:?} from connection {} while being a registering follower",
                            packet,
                            connection_id,
                        );
                    }
                },
                FollowerState::Registered {
                    leader_node_id,
                    leader_connection_id,
                    nodes,
                    connections,
                    ..
                } => match packet {
                    Packet::Ping => {
                        let connection =
                            connections.get(&connection_id).expect("Invalid connection");

                        connection
                            .stream_actor_address
                            .try_send(TcpStreamActorSendMessage::new(
                                Packet::Pong.to_bytes().expect("Cannot write Pong"),
                            ))
                            .expect("Cannot send ActorStopMessage");
                    }
                    Packet::NodeRegisterRequest { .. } => {
                        let node = nodes.get(&leader_node_id).expect("Invalid leader node");
                        let connection = connections
                            .get(&leader_connection_id)
                            .expect("Invalid leader connection");
                        let bytes = Packet::NodeRegisterResponse {
                            result: Err(NodeRegisterResponsePacketError::NotALeader {
                                leader_address: node.socket_address.clone(),
                            }),
                        }
                        .to_bytes()
                        .expect("Cannot write NodeRegisterResponse");

                        connection
                            .stream_actor_address
                            .try_send(TcpStreamActorSendMessage::new(bytes))
                            .expect("Cannot send TcpStreamActorSendMessage");
                    }
                    packet => {
                        error!(
                                "Received invalid packet {:?} from connection {} while being a registered follower",
                                packet,
                                connection_id,
                            );
                    }
                },
            },
            Type::Candidate { .. } => {}
        }
    }
}

impl Handler<NodeActorRemoveConnectionMessage<ConnectionId>> for NodeActor {
    type Result = <NodeActorRemoveConnectionMessage<ConnectionId> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage<ConnectionId>,
        context: &mut <Self as Actor>::Context,
    ) {
        let (connection_id,) = message.into();

        trace!("Destroy connection {}", connection_id);

        match &mut self.r#type {
            Type::Leader {
                ref mut connections,
                ..
            } => {
                connections
                    .remove(&connection_id)
                    .expect("Invalid connection");
            }
            Type::Follower { state, .. } => match state {
                FollowerState::Unregistered => {}
                FollowerState::Registering { .. } => {}
                FollowerState::Registered { .. } => {}
            },
            Type::Candidate {} => {}
        };

        trace!("Destroyed connection {}", connection_id);
    }
}

impl Handler<TcpListenerActorAcceptMessage> for NodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (stream,) = message.into();

        let stream = match stream {
            Ok(stream) => stream,
            Err(error) => {
                error!("{}", error);
                return;
            }
        };

        let peer_address = match stream.peer_addr() {
            Ok(peer_address) => peer_address,
            Err(error) => {
                error!("{}", error);
                return;
            }
        };

        match self.r#type {
            Type::Leader {
                ref mut connections,
                id,
                ..
            } => {
                let connection_id = next_key(&connections);

                trace!("Create connection {}", connection_id);

                let (reader_actor_address, stream_actor_address) = PacketReaderActor::new(
                    connection_id,
                    context.address().recipient(),
                    context.address().recipient(),
                    stream,
                );

                connections.insert(
                    connection_id,
                    LeaderConnection {
                        connection_id,
                        reader_actor_address,
                        stream_actor_address,
                        r#type: None,
                        peer_address,
                    },
                );

                trace!("Connection {} created", connection_id);
            }
            Type::Follower { .. } => {}
            Type::Candidate { .. } => {}
        }
    }
}

impl<T> Handler<LeaderConnectActorMessage<T>> for NodeActor
where
    T: ToSocketAddrs + Debug + Send + Display + 'static,
{
    type Result = ResponseActFuture<Self, <LeaderConnectActorMessage<T> as Message>::Result>;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: LeaderConnectActorMessage<T>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (socket_address,) = message.into();

        trace!("Connect to {}", &socket_address);

        async move { TcpStream::connect(socket_address).await }
            .into_actor(self)
            .map(|result, actor, context| match &mut actor.r#type {
                Type::Follower {
                    ref mut state,
                    listener_local_address,
                    ..
                } => match state {
                    FollowerState::Unregistered => match result {
                        Ok(stream) => {
                            let bytes = Packet::NodeRegisterRequest {
                                local_address: listener_local_address.clone(),
                            }
                            .to_bytes()
                            .expect("Cannot write NodeRegisterRequest");

                            let (reader_actor_address, stream_actor_address) =
                                PacketReaderActor::new(
                                    0 as ConnectionId,
                                    context.address().recipient(),
                                    context.address().recipient(),
                                    stream,
                                );

                            stream_actor_address.do_send(TcpStreamActorSendMessage::new(bytes));

                            *state = FollowerState::Registering {
                                stream_actor_address,
                                reader_actor_address,
                            };
                        }
                        Err(error) => {
                            error!("{}", error);
                        }
                    },
                    state => {
                        error!("Wrong follower state {:?}", state);
                        context.stop();
                    }
                },
                r#type => {
                    error!("Wrong node type {:?}", r#type);
                    context.stop();
                }
            })
            .boxed_local()
    }
}

impl NodeActor {
    pub async fn new<S, T>(
        listener_address: S,
        leader_stream_address: Option<T>,
    ) -> Result<Addr<Self>, io::Error>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs + Debug + Display + Send + Sync + 'static,
    {
        let listener = TcpListener::bind(listener_address).await?;
        let listener_local_address = listener.local_addr()?;

        match leader_stream_address {
            None => Ok(Self::create(|context| Self {
                r#type: Type::Leader {
                    id: 0,
                    listener_actor_address: TcpListenerActor::new(
                        context.address().recipient(),
                        Some(context.address().recipient()),
                        listener,
                    ),
                    listener_local_address,
                    connections: HashMap::default(),
                    nodes: HashMap::default(),
                    heartbeat_spawn_handle: Some(context.run_interval(
                        Duration::new(2, 0),
                        |actor, context| {
                            let connections = match &actor.r#type {
                                Type::Leader {
                                    nodes, connections, ..
                                } => {
                                    for node in nodes.values() {
                                        if matches!(node.state, LeaderNodeState::Registered { .. })
                                        {
                                            let connection = connections
                                                .get(&node.connection_id)
                                                .expect("Invalid connection");
                                            connection
                                                .stream_actor_address
                                                .try_send(TcpStreamActorSendMessage::new(
                                                    Packet::Ping
                                                        .to_bytes()
                                                        .expect("Cannot write Ping"),
                                                ))
                                                .expect("Cannot send Ping");
                                        }
                                    }
                                }
                                _ => {
                                    panic!("Heartbeat should only run for follower node");
                                }
                            };
                        },
                    )),
                },
            })),
            Some(address) => {
                let actor = Self::create(move |context| Self {
                    r#type: Type::Follower {
                        state: FollowerState::Unregistered,
                        listener_actor_address: TcpListenerActor::new(
                            context.address().recipient(),
                            Some(context.address().recipient()),
                            listener,
                        ),
                        listener_local_address,
                    },
                });

                actor.do_send(LeaderConnectActorMessage::new(address));

                Ok(actor)
            }
        }
    }
}
