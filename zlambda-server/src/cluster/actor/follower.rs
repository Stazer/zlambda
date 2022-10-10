use crate::algorithm::next_key;
use crate::cluster::actor::{NodeActor, PacketReaderActor};
use crate::cluster::NodeId;
use crate::cluster::{
    ConnectionId, LeaderConnectActorMessage, NodeActorRegisterMessage,
    NodeActorRemoveConnectionMessage, NodeRegisterResponsePacketError, Packet,
    PacketReaderActorReadPacketMessage, ReadPacketError,NodeRegisterResponsePacketSuccessNodeData
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
struct LeaderNode {
    id: NodeId,
    socket_address: SocketAddr,
    connection: Weak<Connection>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct FollowerNode {
    id: NodeId,
    socket_address: SocketAddr,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum Node {
    Leader(Rc<LeaderNode>),
    Follower(FollowerNode)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum ConnectionType {
    Client,
    Leader(Weak<LeaderNode>),
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
        reader_address: Addr<PacketReaderActor<()>>,
    },
    Registered {
        node_id: NodeId,
        leader_node: Weak<LeaderNode>,
        nodes: HashMap<NodeId, Rc<Node>>,
        connections: HashMap<ConnectionId, Rc<Connection>>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerNodeActor {
    node_address: Addr<NodeActor>,
    listener_address: Addr<TcpListenerActor>,
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
                Rc::new(Connection {
                    id,
                    reader_address,
                    stream_address,
                    r#type: None,
                }),
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

        let (connection_id, packet): (ConnectionId, Packet) = message.into();

        let connection = connections
            .get(&connection_id)
            .expect("Connection not found");

        /*match (&self.state, packet) {
            (State::Unregistered {}, Packet::NodeRegisterResponse { result }) => {
                match result {
                    Ok(o) => {}
                    Err(NodeRegisterResponsePacketError::NotRegistered) => {
                        context.stop()
                    }
                    Err(NodeRegisterResponsePacketError::NotALeader { leader_address }) => {
                        context.notify(LeaderConnectActorMessage::new(leader_address))
                    }
                }
            }
            (State::Unregistered {}, Packet::NodeRegisterRequest) => {
                connection
                    .stream_address
                    .do_send(TcpStreamActorSendMessage::new(
                        (Packet::NodeRegisterResponse {
                            result: Err(NodeRegisterResponsePacketError::NotRegistered),
                        })
                        .to_bytes()
                        .expect("Cannot write NodeRegisterResponse"),
                    ))
            }
            (State::Registered { leader_node, .. }, Packet::NodeRegisterRequest) => {
                    connection
                        .stream_address
                        .do_send(TcpStreamActorSendMessage::new(
                            (Packet::NodeRegisterResponse {
                                result: Err(NodeRegisterResponsePacketError::NotALeader {
                                    leader_address: leader_node.socket_address.clone(),
                                }),
                            })
                            .to_bytes()
                            .expect("Cannot write NodeRegisterResponse"),
                        ))

            }
            (State::Registered { .. }, Packet::NodePing) => connection
                .stream_address
                .do_send(TcpStreamActorSendMessage::new(
                    (Packet::NodePong)
                        .to_bytes()
                        .expect("Cannot write NodePong"),
                )),
            (_, packet) => {
                error!(
                    "Unhandled packet {:?} by connection {}",
                    packet, connection_id
                );

                connection.stream_address.do_send(ActorStopMessage);
                connection.reader_address.do_send(ActorStopMessage);

                assert!(self.connections.remove(&connection_id).is_none())
            }
        }*/
    }
}

impl Handler<PacketReaderActorReadPacketMessage<()>> for FollowerNodeActor {
    type Result = <PacketReaderActorReadPacketMessage<()> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage<()>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (stream_address, reader_address) = match &self.state {
            (State::Unregistered | State::Registered { .. }) => {
                error!("Node is not registering");
                return;
            },
            State::Registering { stream_address, reader_address } => (stream_address.clone(), reader_address.clone()),
        };

        let (_, packet): ((), Packet) = message.into();

        match packet {
            Packet::NodeRegisterResponse { result } => {
                match result {
                    Ok(o) => {
                        let (node_id, nodes, leader_node_id): (NodeId, Vec<NodeRegisterResponsePacketSuccessNodeData>, NodeId) = o.into();

                        let socket_addresses = nodes.into_iter().map(|node| {
                            let (node_id, socket_address) = node.into();

                            (node_id, socket_address)
                        }).collect::<HashMap::<_, _>>();

                        let mut leader_node = Rc::new(LeaderNode {
                            id: leader_node_id,
                            socket_address: socket_addresses.remove(&leader_node_id).expect(""),
                            connection: Weak::new(),
                        });

                        let mut connections = HashMap::default();
                        let connection_id = next_key(&connections);
                        let connection = Rc::new(Connection {
                            id: connection_id,
                            stream_address,
                            reader_address,
                            r#type: Some(ConnectionType::Leader(Rc::downgrade(&leader_node))),
                        });
                        leader_node.connection = Rc::downgrade(&connection);
                        connections.insert(connection_id, connection);

                        let mut nodes = HashMap::default();
                        nodes.insert(leader_node_id, Rc::new(Node::Leader(leader_node)));

                        self.state = State::Registered {
                            node_id,
                            leader_node: Rc::downgrade(&leader_node),
                            nodes,
                            connections,
                        };
                    },
                    Err(NodeRegisterResponsePacketError::NotALeader { leader_address }) => {
                        error!("Not a leader");
                        self.state = State::Unregistered;
                        stream_address.do_send(ActorStopMessage);
                        reader_address.do_send(ActorStopMessage);
                        context.address().do_send(LeaderConnectActorMessage::new(leader_address))
                    },
                    Err(NodeRegisterResponsePacketError::NotRegistered) => {
                        error!("Not registered");
                        self.state = State::Unregistered;
                        context.stop()
                    },
                }
            }
            packet => {
                error!(
                    "Unhandled packet {:?}",
                    packet
                );

                stream_address.do_send(ActorStopMessage);
                reader_address.do_send(ActorStopMessage);
                self.state = State::Unregistered;

                context.stop()
            }
        }
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

        let packet = Packet::NodeRegisterRequest
            .to_bytes()
            .expect("Cannot create NodeRegisterRequest");

        async move { TcpStream::connect(socket_address).await }
            .into_actor(self)
            .map(move |result, actor, context| match result {
                Ok(stream) => {
                    let (reader_address, stream_address) = PacketReaderActor::new(
                        (),
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
        let actor = Self::create(|context| Self {
            node_address,
            listener_address: TcpListenerActor::new(
                context.address().recipient(),
                Some(context.address().recipient()),
                listener,
            ),
            state: State::Unregistered,
        });

        actor.do_send(LeaderConnectActorMessage::new(leader_stream_address));

        actor
    }
}
