use crate::algorithm::next_key;
use crate::cluster::actor::{NodeActor, PacketReaderActor};
use crate::cluster::NodeId;
use crate::cluster::{
    ConnectionId, LeaderConnectActorMessage, NodeActorRegisterMessage,
    NodeActorRemoveConnectionMessage, NodeRegisterResponsePacketError,
    NodeRegisterResponsePacketSuccessNodeData, Packet, PacketReaderActorReadPacketMessage,
    ReadPacketError,
};
use crate::common::{
    ActorExecuteMessage, ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage,
    TcpStreamActor, TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, Recipient,
    ResponseActFuture, WrapFuture,
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::io;
use std::net::SocketAddr;
use std::rc::{Rc, Weak};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Node {
    id: NodeId,
    socket_address: SocketAddr,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum ConnectionType {
    Client,
    Leader(NodeId),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Connection {
    id: ConnectionId,
    stream_address: Addr<TcpStreamActor>,
    reader_address: Addr<PacketReaderActor<ConnectionId>>,
    r#type: Option<ConnectionType>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum State {
    Unregistered,
    Registering {
        stream_address: Addr<TcpStreamActor>,
        reader_address: Addr<PacketReaderActor<ConnectionId>>,
    },
    Registered {
        node_id: NodeId,
        leader_node_id: NodeId,
        leader_connection_id: ConnectionId,
        nodes: HashMap<NodeId, Node>,
        connections: HashMap<ConnectionId, Connection>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerNodeActor {
    node_address: Addr<NodeActor>,
    listener_address: Addr<TcpListenerActor>,
    local_address: SocketAddr,
    state: State,
}

impl Actor for FollowerNodeActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _context: &mut Self::Context) {
        self.node_address.do_send(ActorStopMessage);
        self.listener_address.do_send(ActorStopMessage);

        match &self.state {
            State::Unregistered => {}
            State::Registering {
                stream_address,
                reader_address,
            } => {
                stream_address.do_send(ActorStopMessage);
                reader_address.do_send(ActorStopMessage);
            }
            State::Registered { connections, .. } => {
                for connection in connections.values() {
                    connection.stream_address.do_send(ActorStopMessage);
                    connection.reader_address.do_send(ActorStopMessage);
                }
            }
        };
    }
}

impl Handler<ActorStopMessage> for FollowerNodeActor {
    type Result = <ActorStopMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(&mut self, _message: ActorStopMessage, context: &mut <Self as Actor>::Context) {
        context.stop();
    }
}

impl Handler<TcpListenerActorAcceptMessage> for FollowerNodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        trace!("Create connection");

        let stream = match message.into() {
            (Err(e),) => {
                error!("{}", e);
                return;
            }
            (Ok(s),) => s,
        };

        let (connections,) = match &mut self.state {
            State::Unregistered | State::Registering { .. } => {
                error!("Follower node is not registered");
                return;
            }
            State::Registered {
                ref mut connections,
                ..
            } => (connections,),
        };

        let id = next_key(&connections);

        let (reader_address, stream_address) = PacketReaderActor::new(
            id,
            context.address().recipient(),
            context.address().recipient(),
            stream,
        );

        assert!(connections
            .insert(
                id,
                Connection {
                    id,
                    reader_address,
                    stream_address,
                    r#type: None,
                },
            )
            .is_none());

        trace!("Connection {} created", id);
    }
}

impl Handler<NodeActorRemoveConnectionMessage<ConnectionId>> for FollowerNodeActor {
    type Result = <NodeActorRemoveConnectionMessage<ConnectionId> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage<ConnectionId>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (connections,) = match self.state {
            State::Unregistered | State::Registering { .. } => {
                error!("Follower node is not registered");
                return;
            }
            State::Registered {
                ref mut connections,
                ..
            } => (connections,),
        };

        trace!("Destroy connection {}", message.connection_id());

        let connection = connections
            .get(&message.connection_id())
            .expect("Connection not found");

        connection.stream_address.do_send(ActorStopMessage);
        connection.reader_address.do_send(ActorStopMessage);

        assert!(connections.remove(&message.connection_id()).is_none());

        trace!("Connection destroyed");
    }
}

impl Handler<PacketReaderActorReadPacketMessage<ConnectionId>> for FollowerNodeActor {
    type Result = <PacketReaderActorReadPacketMessage<ConnectionId> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage<ConnectionId>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (connection_id, packet): (ConnectionId, Packet) = message.into();

        match self.state {
            State::Unregistered => {
                error!(
                    "Unhandled packet {:?} from connection {} while being unregistered",
                    packet, connection_id,
                );

                context.stop();
            }
            State::Registering {
                ref stream_address,
                ref reader_address,
            } => match packet {
                Packet::NodeRegisterResponse { result } => match result {
                    Ok(data) => {
                        let (node_id, nodes, leader_node_id): (
                            NodeId,
                            Vec<NodeRegisterResponsePacketSuccessNodeData>,
                            NodeId,
                        ) = data.into();

                        let nodes = nodes
                            .into_iter()
                            .map(|node| {
                                let (node_id, socket_address) = node.into();

                                (
                                    node_id,
                                    Node {
                                        id: node_id,
                                        socket_address,
                                    },
                                )
                            })
                            .collect::<HashMap<_, _>>();

                        let mut connections = HashMap::default();
                        connections.insert(
                            connection_id,
                            Connection {
                                id: connection_id,
                                stream_address: stream_address.clone(),
                                reader_address: reader_address.clone(),
                                r#type: Some(ConnectionType::Leader(leader_node_id)),
                            },
                        );

                        self.state = State::Registered {
                            node_id,
                            leader_node_id,
                            leader_connection_id: connection_id,
                            nodes,
                            connections,
                        };
                    }
                    Err(NodeRegisterResponsePacketError::NotALeader { leader_address }) => {
                        stream_address.do_send(ActorStopMessage);
                        reader_address.do_send(ActorStopMessage);
                        context
                            .address()
                            .do_send(LeaderConnectActorMessage::new(leader_address));

                        self.state = State::Unregistered;

                        error!("Not a leader");
                    }
                    Err(NodeRegisterResponsePacketError::NotRegistered) => {
                        stream_address.do_send(ActorStopMessage);
                        reader_address.do_send(ActorStopMessage);

                        self.state = State::Unregistered;

                        error!("Not registered");
                    }
                },
                _ => {
                    error!(
                        "Unhandled packet {:?} from connection {} while registering",
                        packet, connection_id,
                    );
                }
            },
            State::Registered {
                ref connections, ..
            } => match packet {
                Packet::NodePing => {
                    connections
                        .get(&connection_id)
                        .expect("Connection not found")
                        .stream_address
                        .send(TcpStreamActorSendMessage::new(
                            Packet::NodePong.to_bytes().expect("Cannot write NodePong"),
                        ));
                }
                _ => {
                    error!(
                        "Unhandled packet {:?} from connection {}",
                        packet, connection_id,
                    );
                }
            },
        };
    }
}

impl Handler<NodeActorRemoveConnectionMessage<()>> for FollowerNodeActor {
    type Result = <NodeActorRemoveConnectionMessage<()> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage<()>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        //let (_, packet): ((), Packet) = message.into();
    }
}

impl<T> Handler<LeaderConnectActorMessage<T>> for FollowerNodeActor
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

        match self.state {
            State::Unregistered => {}
            State::Registering { .. } => {
                error!("Node is registering");
            }
            State::Registered { .. } => {
                error!("Node is registered");
            }
        };
        trace!("Connect to {}", &socket_address);

        let packet = Packet::NodeRegisterRequest {
            local_address: self.local_address
        }
            .to_bytes()
            .expect("Cannot create NodeRegisterRequest");

        async move { TcpStream::connect(socket_address).await }
            .into_actor(self)
            .map(move |result, actor, context| match result {
                Ok(stream) => {
                    let (reader_address, stream_address) = PacketReaderActor::new(
                        0 as ConnectionId,
                        context.address().recipient(),
                        context.address().recipient(),
                        stream,
                    );

                    actor.state = State::Registering {
                        stream_address: stream_address.clone(),
                        reader_address,
                    };

                    stream_address.do_send(TcpStreamActorSendMessage::new(packet));
                }
                Err(e) => {
                    context.stop();
                    error!("Cannot connect ({})", e);
                }
            })
            .boxed_local()
    }
}

impl FollowerNodeActor {
    pub fn new<T>(
        node_address: Addr<NodeActor>,
        listener: TcpListener,
        leader_stream_address: T,
    ) -> Addr<Self>
    where
        T: ToSocketAddrs + Debug + Send + Display + 'static,
    {
        let local_address = listener.local_addr().expect("Cannot read local address");

        let actor = Self::create(|context| Self {
            node_address,
            listener_address: TcpListenerActor::new(
                context.address().recipient(),
                Some(context.address().recipient()),
                listener,
            ),
            local_address,
            state: State::Unregistered,
        });

        actor.do_send(LeaderConnectActorMessage::new(leader_stream_address));

        actor
    }
}
