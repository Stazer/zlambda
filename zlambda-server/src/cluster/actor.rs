use crate::algorithm::next_key;
use crate::cluster::NodeId;
use crate::cluster::{
    ConnectionId, NodeActorRegisterMessage, NodeActorRemoveConnectionMessage,
    NodeRegisterResponsePacketError, Packet, PacketReaderActorReadPacketMessage, ReadPacketError,
};
use crate::common::{
    ActorExecuteMessage, ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage,
    TcpStreamActor, TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message,
    ResponseActFuture, WrapFuture,
};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

type MessageId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum MessageNodeState {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct NodeNodeState {
    connection_id: Option<ConnectionId>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum ConnectionTypeState {
    IncomingRegisteringNode,
    OutgoingRegisteringNode,
    Node { node_id: NodeId },
    Client,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ConnectionNodeState {
    id: ConnectionId,
    r#type: Option<ConnectionTypeState>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct LeaderNodeNodeState {
    connection_id: ConnectionId,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct FollowerNodeNodeState {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum NodeTypeState {
    Leader {
        node_id: NodeId,
        //nodes: HashMap<NodeId, NodeNodeState>,
    },
    Follower {
        node_id: NodeId,
        //nodes: HashMap<NodeId, NodeNodeState>
    },
    Candidate,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct NodeState {
    r#type: Option<NodeTypeState>,
    connections: HashMap<ConnectionId, ConnectionNodeState>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActor {
    listener_address: Addr<TcpListenerActor>,
    stream_addresses: HashMap<ConnectionId, Addr<TcpStreamActor>>,
    reader_addresses: HashMap<ConnectionId, Addr<PacketReaderActor>>,
    state: NodeState,
}

impl Actor for NodeActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _context: &mut Self::Context) {
        self.listener_address.do_send(ActorStopMessage);

        for stream in self.stream_addresses.values() {
            stream.do_send(ActorStopMessage);
        }

        for reader in self.reader_addresses.values() {
            reader.do_send(ActorStopMessage);
        }
    }
}

impl Handler<ActorStopMessage> for NodeActor {
    type Result = <ActorStopMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
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
        trace!("Create connection");

        let stream = match message.into() {
            (Err(e),) => {
                error!("{}", e);
                return;
            }
            (Ok(s),) => s,
        };

        let connection_id = next_key(&self.state.connections);
        assert!(self
            .state
            .connections
            .insert(
                connection_id,
                ConnectionNodeState {
                    id: connection_id,
                    r#type: None,
                }
            )
            .is_none());

        let (reader_address, stream_address) =
            PacketReaderActor::new(connection_id, context.address(), stream);
        self.stream_addresses.insert(connection_id, stream_address);
        self.reader_addresses.insert(connection_id, reader_address);

        trace!("Connection {} created", connection_id);
    }
}

impl Handler<NodeActorRemoveConnectionMessage> for NodeActor {
    type Result = <NodeActorRemoveConnectionMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        trace!("Destroy connection {}", message.connection_id());

        self.stream_addresses.remove(&message.connection_id());
        self.reader_addresses.remove(&message.connection_id());
        assert!(self
            .state
            .connections
            .remove(&message.connection_id())
            .is_some());

        trace!("Connection destroyed");
    }
}

impl<F> Handler<ActorExecuteMessage<F, NodeActor>> for NodeActor
where
    F: FnOnce(&mut NodeActor, &mut <NodeActor as Actor>::Context),
{
    type Result = <ActorExecuteMessage<F, NodeActor> as Message>::Result;

    fn handle(
        &mut self,
        message: ActorExecuteMessage<F, NodeActor>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (function,) = message.into();
        function(self, context);
    }
}

impl Handler<PacketReaderActorReadPacketMessage> for NodeActor {
    type Result = <PacketReaderActorReadPacketMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (connection_id, packet): (ConnectionId, Packet) = message.into();

        match packet {
            Packet::NodeRegisterRequest => {
                let connection = self
                    .state
                    .connections
                    .get_mut(&connection_id)
                    .expect("Connection not found");
            }
            Packet::NodeRegisterResponse { result } => {
                match result {
                    Ok(node_id) => {
                        assert!(self.state.r#type.is_none());

                        println!("Registration successful");

                        self.state.r#type = Some(NodeTypeState::Follower {
                            node_id,
                            //leader_connection_id: connection_id,
                        });
                    }
                    Err(NodeRegisterResponsePacketError::NotALeader { leader_address }) => {
                        let stream_address = self
                            .stream_addresses
                            .get(&connection_id)
                            .expect("Reader not found")
                            .clone();

                        let reader_address = self
                            .reader_addresses
                            .get(&connection_id)
                            .expect("Reader not found")
                            .clone();

                        context.wait(
                            async move {
                                if let Err(e) = stream_address.send(ActorStopMessage).await {
                                    return Err(Box::<dyn Error>::from(e));
                                }

                                reader_address
                                    .send(ActorStopMessage)
                                    .await
                                    .map_err(Box::<dyn Error>::from)
                            }
                            .into_actor(self)
                            .map(|result, _actor, context| {
                                if let Err(e) = result {
                                    error!("{}", e);
                                    context.stop();
                                }
                            }),
                        );
                    }
                }
            }
            _ => {
                let stream_address = self
                    .stream_addresses
                    .get(&connection_id)
                    .expect("Reader not found")
                    .clone();

                context.wait(
                    async move { stream_address.send(ActorStopMessage).await }
                        .into_actor(self)
                        .map(|result, _actor, context| {
                            if let Err(e) = result {
                                error!("{}", e);
                                context.stop();
                            }
                        }),
                );
            }
        };
    }
}

impl Handler<NodeActorRegisterMessage> for NodeActor {
    type Result = ResponseActFuture<Self, <NodeActorRegisterMessage as Message>::Result>;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRegisterMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (stream,): (TcpStream,) = message.into();

        let connection_id = next_key(&self.state.connections);
        assert!(self.state.r#type.is_none());
        assert!(self
            .state
            .connections
            .insert(
                connection_id,
                ConnectionNodeState {
                    id: connection_id,
                    r#type: Some(ConnectionTypeState::OutgoingRegisteringNode),
                }
            )
            .is_none());

        let (reader_address, stream_address) =
            PacketReaderActor::new(connection_id, context.address(), stream);
        self.stream_addresses
            .insert(connection_id, stream_address.clone());
        self.reader_addresses.insert(connection_id, reader_address);

        async move {
            let packet = match Packet::NodeRegisterRequest.to_bytes() {
                Err(e) => return Err(Box::<dyn Error>::from(e)),
                Ok(p) => p,
            };

            match stream_address
                .send(TcpStreamActorSendMessage::new(packet))
                .await
            {
                Err(e) => Err(Box::<dyn Error>::from(e)),
                Ok(Err(e)) => Err(Box::<dyn Error>::from(e)),
                Ok(Ok(())) => Ok(()),
            }
        }
        .into_actor(self)
        .map(|result, _actor, context| {
            if let Err(error) = result {
                error!("{}", error);
                context.stop();
            }
        })
        .boxed_local()
    }
}

impl NodeActor {
    pub async fn new<S, T>(
        listener_address: S,
        stream_address: Option<T>,
    ) -> Result<Addr<Self>, io::Error>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let stream = match stream_address {
            Some(s) => Some(TcpStream::connect(s).await?),
            None => None,
        };

        let listener = TcpListener::bind(listener_address).await?;

        let is_leader = stream.is_none();

        let actor = Self::create(move |context| Self {
            listener_address: TcpListenerActor::new(
                context.address().recipient(),
                Some(context.address().recipient()),
                listener,
            ),
            reader_addresses: HashMap::default(),
            stream_addresses: HashMap::default(),
            state: if is_leader {
                NodeState {
                    r#type: Some(NodeTypeState::Leader { node_id: 0 }),
                    connections: HashMap::new(),
                }
            } else {
                NodeState {
                    r#type: None,
                    connections: HashMap::new(),
                }
            },
        });

        if let Some(stream) = stream {
            actor
                .send(NodeActorRegisterMessage::new(stream))
                .await
                .expect("Cannot send NodeActorRegisterMessage");
        }

        Ok(actor)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct PacketReaderActor {
    id: ConnectionId,
    recipient: Addr<NodeActor>,
    stream: Addr<TcpStreamActor>,
    buffer: Vec<u8>,
}

impl Actor for PacketReaderActor {
    type Context = Context<Self>;

    #[tracing::instrument]
    fn stopped(&mut self, _: &mut Self::Context) {
        self.stream.do_send(ActorStopMessage);
        self.recipient
            .do_send(NodeActorRemoveConnectionMessage::new(self.id));
    }
}

impl Handler<ActorStopMessage> for PacketReaderActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl Handler<TcpStreamActorReceiveMessage> for PacketReaderActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result,) = message.into();

        let bytes = match result {
            Ok(b) => b,
            Err(e) => {
                error!("{:?}", e);
                context.stop();
                return;
            }
        };

        self.buffer.extend(bytes);

        loop {
            let (read, packet) = match Packet::from_vec(&self.buffer) {
                Ok((r, p)) => (r, p),
                Err(ReadPacketError::UnexpectedEnd) => break,
                Err(e) => {
                    error!("{:?}", e);
                    context.stop();
                    return;
                }
            };

            self.buffer.drain(0..read);

            self.recipient
                .do_send(PacketReaderActorReadPacketMessage::new(self.id, packet));
        }
    }
}

impl PacketReaderActor {
    pub fn new(
        id: ConnectionId,
        recipient: Addr<NodeActor>,
        stream: TcpStream,
    ) -> (Addr<Self>, Addr<TcpStreamActor>) {
        let context = Context::new();
        let stream = TcpStreamActor::new(
            context.address().recipient(),
            Some(context.address().recipient()),
            stream,
        );

        (
            context.run(Self {
                id,
                recipient,
                stream: stream.clone(),
                buffer: Vec::default(),
            }),
            stream,
        )
    }
}
