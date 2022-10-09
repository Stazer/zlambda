use crate::algorithm::next_key;
use crate::cluster::actor::{NodeActor, PacketReaderActor};
use crate::cluster::NodeId;
use crate::cluster::{
    ConnectionId, LeaderConnectActorMessage, NodeActorRegisterMessage,
    NodeActorRemoveConnectionMessage, NodeRegisterResponsePacketError, Packet,
    PacketReaderActorReadPacketMessage, ReadPacketError,
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
use std::rc::Rc;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct FollowerNodeLeaderNode {
    node_id: NodeId,
    socket_address: SocketAddr,
    connection_id: ConnectionId,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct FollowerNodeFollowerNode {
    node_id: NodeId,
    socket_address: SocketAddr,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum FollowerNodeNode {
    Leader(Rc<FollowerNodeLeaderNode>),
    Follower(FollowerNodeFollowerNode),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum FollowerNodeConnectionType {
    Registration,
    Client,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct FollowerNodeConnection {
    connection_id: ConnectionId,
    stream_address: Addr<TcpStreamActor>,
    reader_address: Addr<PacketReaderActor<ConnectionId>>,
    r#type: Option<FollowerNodeConnectionType>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum FollowerNodeState {
    Unregistered {
    },
    Registered {
        node_id: NodeId,
        leader_node: Rc<FollowerNodeLeaderNode>,
        nodes: HashMap<NodeId, FollowerNodeNode>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerNodeActor {
    node_address: Addr<NodeActor>,
    listener_address: Addr<TcpListenerActor>,
    connections: HashMap<ConnectionId, FollowerNodeConnection>,
    state: FollowerNodeState,
}

impl Actor for FollowerNodeActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _context: &mut Self::Context) {
        self.node_address.do_send(ActorStopMessage);
        self.listener_address.do_send(ActorStopMessage);

        for connection in self.connections.values() {
            connection.stream_address.do_send(ActorStopMessage);
            connection.reader_address.do_send(ActorStopMessage);
        }
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

        let connection_id = next_key(&self.connections);

        let (reader_address, stream_address) = PacketReaderActor::new(
            connection_id,
            context.address().recipient(),
            context.address().recipient(),
            stream,
        );

        assert!(self
            .connections
            .insert(
                connection_id,
                FollowerNodeConnection {
                    connection_id,
                    reader_address,
                    stream_address,
                    r#type: None,
                }
            )
            .is_none());

        trace!("Connection {} created", connection_id);
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
        trace!("Destroy connection {}", message.connection_id());

        let connection = self
            .connections
            .get(&message.connection_id())
            .expect("Connection not found");

        connection.stream_address.do_send(ActorStopMessage);
        connection.reader_address.do_send(ActorStopMessage);

        assert!(self.connections.remove(&message.connection_id()).is_none());

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

        let connection = self
            .connections
            .get(&connection_id)
            .expect("Connection not found");

        match (&self.state, packet) {
            (FollowerNodeState::Unregistered {}, Packet::NodeRegisterResponse { result }) => {
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
            (FollowerNodeState::Unregistered {}, Packet::NodeRegisterRequest) => {
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
            (FollowerNodeState::Registered { leader_node, .. }, Packet::NodeRegisterRequest) => {
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
            (FollowerNodeState::Registered { .. }, Packet::NodePing) => connection
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
        }
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

        let packet = Packet::NodeRegisterRequest
            .to_bytes()
            .expect("Cannot create NodeRegisterRequest");

        async move { TcpStream::connect(socket_address).await }
            .into_actor(self)
            .map(move |result, actor, context| match result {
                Ok(stream) => {
                    let connection_id = next_key(&actor.connections);

                    let (reader_address, stream_address) = PacketReaderActor::new(
                        connection_id,
                        context.address().recipient(),
                        context.address().recipient(),
                        stream,
                    );

                    assert!(actor
                        .connections
                        .insert(
                            connection_id,
                            FollowerNodeConnection {
                                connection_id,
                                reader_address,
                                stream_address: stream_address.clone(),
                                r#type: Some(FollowerNodeConnectionType::Registration),
                            }
                        )
                        .is_none());

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
            connections: HashMap::default(),
            state: FollowerNodeState::Unregistered {},
        });

        actor.do_send(LeaderConnectActorMessage::new(leader_stream_address));

        actor
    }
}
