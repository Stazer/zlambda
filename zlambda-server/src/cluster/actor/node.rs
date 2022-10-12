use crate::algorithm::next_key;
use crate::cluster::NodeId;
use crate::cluster::{
    ConnectionId, LeaderConnectActorMessage, NodeActorRegisterMessage,
    NodeActorRemoveConnectionMessage, NodeRegisterResponsePacketError, Packet,
    PacketReaderActorReadPacketMessage, ReadPacketError,
    NodeRegisterResponsePacketSuccessNodeData,
};
use crate::cluster::actor::{PacketReaderActor};
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
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::net::SocketAddr;
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct FollowerNode {
    id: NodeId,
    socket_address: SocketAddr,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum LeaderConnectionType {
    Node(NodeId)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderConnection {
    id: ConnectionId,
    r#type: Option<LeaderConnectionType>,
    stream_actor_address: Addr<TcpStreamActor>,
    reader_actor_address: Addr<PacketReaderActor<ConnectionId>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderNode {
    connection_id: ConnectionId,
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
        id: NodeId,
        nodes: HashMap<NodeId, FollowerNode>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum Type {
    Leader {
        id: NodeId,
        connections: HashMap<ConnectionId, LeaderConnection>,
        nodes: HashMap<NodeId, LeaderNode>,
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

    fn stopped(&mut self, _context: &mut Self::Context) {
        /*match self.r#type {
            Type::Leader {} => {
            }
            Type::Follower {} => {
            }
            Type ::Candidate {} => {}
        }*/
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
    fn handle(&mut self, message: PacketReaderActorReadPacketMessage<ConnectionId>, context: &mut <Self as Actor>::Context) {
        let (connection_id, packet) = message.into();

        match &mut self.r#type {
            Type::Leader {..} => {

            },
            Type::Follower { ref mut state, .. } => {
                match state {
                    FollowerState::Unregistered => {
                        match packet {
                            packet => {
                                error!("Received invalid packet {:?} while being an unregistered follower", packet);
                            }
                        }
                    }
                    FollowerState::Registering { stream_actor_address, reader_actor_address } => {
                        match packet {
                            Packet::NodeRegisterResponse { result } => {
                                match result {
                                    Ok(data) => {
                                        let (node_id, nodes, leader_node_id): (
                                            NodeId,
                                            Vec<NodeRegisterResponsePacketSuccessNodeData>,
                                            NodeId,
                                        ) = data.into();


                                    },
                                    Err(NodeRegisterResponsePacketError::NotALeader { leader_address }) => {
                                        stream_actor_address.do_send(ActorStopMessage);
                                        reader_actor_address.do_send(ActorStopMessage);

                                        *state = FollowerState::Unregistered;

                                        error!("Not a leader");
                                    }
                                    Err(NodeRegisterResponsePacketError::NotRegistered) => {
                                        stream_actor_address.do_send(ActorStopMessage);
                                        reader_actor_address.do_send(ActorStopMessage);

                                        *state = FollowerState::Unregistered;

                                        error!("Not registered");
                                    }
                                }
                            }

                            packet => {
                                stream_actor_address.do_send(ActorStopMessage);
                                reader_actor_address.do_send(ActorStopMessage);

                                error!("Received invalid packet {:?} while being a registering follower", packet);
                            }
                        }
                    }
                    FollowerState::Registered { .. } => {

                    }
                }
            }
            Type::Candidate { .. } => {

            }
        }
    }
}

impl Handler<NodeActorRemoveConnectionMessage<ConnectionId>> for NodeActor {
    type Result = <NodeActorRemoveConnectionMessage<ConnectionId> as Message>::Result;

    #[tracing::instrument]
    fn handle(&mut self, message: NodeActorRemoveConnectionMessage<ConnectionId>, context: &mut <Self as Actor>::Context) {
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
        let (stream, ) = message.into();

        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        match self.r#type {
            Type::Leader { ref mut connections, id, .. }  => {
                let connection_id = next_key(&connections);

                let (reader_actor_address, stream_actor_address) = PacketReaderActor::new(
                    connection_id,
                    context.address().recipient(),
                    context.address().recipient(),
                    stream,
                );

                connections.insert(connection_id, LeaderConnection {
                    id: connection_id,
                    r#type: None,
                    reader_actor_address,
                    stream_actor_address
                });

                trace!("Connection {} created", connection_id);
            },
            Type::Follower { .. } => {
            },
            Type::Candidate { ..} => {

            },
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

        async move {
            TcpStream::connect(socket_address).await
        }
        .into_actor(self)
            .map(|result, actor, context| {
                match &mut actor.r#type {
                    Type::Follower { ref mut state, listener_local_address, .. } => match state {
                        FollowerState::Unregistered => {
                            match result {
                                Ok(stream) => {
                                    let bytes = match (Packet::NodeRegisterRequest {
                                        local_address: listener_local_address.clone(),
                                    }.to_bytes()) {
                                        Ok(bytes) => bytes,
                                        Err(e) => {
                                            error!("{}", e);
                                            return;
                                        }
                                    };

                                    let (reader_actor_address, stream_actor_address) = PacketReaderActor::new(
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
                            }
                        }
                        state => {
                            error!(
                                "Wrong follower state {:?}",
                                state
                            );
                            context.stop();
                        }
                    },
                    r#type => {
                        error!(
                            "Wrong node type {:?}",
                            r#type
                        );
                        context.stop();
                    }
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
                    connections: HashMap::default(),
                    nodes: HashMap::default(),
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
