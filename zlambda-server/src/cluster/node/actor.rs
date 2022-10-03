use crate::cluster::{
    ConnectionId, ConsensusActor, ConsensusActorBroadcastMessage,
    ConsensusActorSendBeginRequestMessage, ConsensusActorSendBeginResponseMessage,
    ConsensusActorSendCommitRequestMessage, ConsensusActorSendCommitResponseMessage,
    NodeActorRegisterMessage, NodeActorRemoveConnectionMessage, Packet, PacketReaderActor,
    PacketReaderActorReadPacketMessage, StateActor, StateActorCreateConnectionMessage,
    StateActorDestroyConnectionMessage, StateActorReadDataMessage, StateActorCreateNodeConnectionMessage
};
use crate::common::{
    ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor,
    TcpStreamActorSendMessage,
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

#[derive(Debug)]
pub enum NodeMessage {
    RegisterNode { connection_id: ConnectionId },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActor {
    consensus: Addr<ConsensusActor<NodeMessage, Self>>,
    state: Addr<StateActor>,
    listener: Addr<TcpListenerActor>,
    streams: HashMap<ConnectionId, Addr<TcpStreamActor>>,
    readers: HashMap<ConnectionId, Addr<PacketReaderActor>>,
}

impl Actor for NodeActor {
    type Context = Context<Self>;

    #[tracing::instrument]
    fn stopped(&mut self, _context: &mut Self::Context) {
        self.state.do_send(ActorStopMessage);
        self.listener.do_send(ActorStopMessage);

        for stream in self.streams.values() {
            stream.do_send(ActorStopMessage);
        }

        for reader in self.readers.values() {
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

impl Handler<PacketReaderActorReadPacketMessage> for NodeActor {
    type Result = ResponseActFuture<Self, <PacketReaderActorReadPacketMessage as Message>::Result>;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        async {}
            .into_actor(self)
            .then(|_, actor, _context| {
                let (connection_id, packet): (ConnectionId, Packet) = message.into();
                let stream = actor.streams.get(&connection_id).map(|x| x.clone());
                let _state = actor.state.clone();
                let consensus = actor.consensus.clone();

                async move {
                    let stream = match stream {
                        None => return Err(Box::<dyn Error>::from("Stream not found")),
                        Some(s) => s,
                    };

                    match packet {
                        Packet::NodeRegisterRequest => {
                            if let Err(e) = consensus
                                .send(ConsensusActorBroadcastMessage::new(
                                    NodeMessage::RegisterNode { connection_id },
                                ))
                                .await
                            {
                                return Err(Box::<dyn Error>::from(e));
                            }
                        }
                        Packet::NodeRegisterResponse { result } => {
                            trace!("{:?}", result);
                        }
                        _ => {
                            error!("Unhandled packet from connection {}", connection_id);

                            if let Err(e) = stream.send(ActorStopMessage).await {
                                return Err(Box::<dyn Error>::from(e));
                            }
                        }
                    }

                    Ok(())
                }
                .into_actor(actor)
            })
            .map(|result, _actor, context| {
                if let Err(e) = result {
                    error!("{}", e);
                    context.stop();
                }
            })
            .boxed_local()
    }
}

impl Handler<NodeActorRemoveConnectionMessage> for NodeActor {
    type Result = ResponseActFuture<Self, <NodeActorRemoveConnectionMessage as Message>::Result>;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (id,): (ConnectionId,) = message.into();

        trace!("Destroy connection {}", id);

        assert!(self
            .streams
            .remove(&id)
            .map(|r| {
                r.do_send(ActorStopMessage);
                r
            })
            .is_some());
        assert!(self
            .readers
            .remove(&id)
            .map(|r| {
                r.do_send(ActorStopMessage);
                r
            })
            .is_some());

        let future = self.state.send(StateActorDestroyConnectionMessage::new(id));

        Box::pin(
            async move { future.await }
                .into_actor(self)
                .map(|result, _actor, _context| {
                    match result {
                        Err(e) => {
                            error!("{}", e);
                            return;
                        }
                        Ok(()) => {}
                    };

                    trace!("Connection destroyed");
                }),
        )
    }
}

impl Handler<TcpListenerActorAcceptMessage> for NodeActor {
    type Result = ResponseActFuture<Self, <TcpListenerActorAcceptMessage as Message>::Result>;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        trace!("Create connection");

        let stream = match message.into() {
            (Err(e),) => {
                eprintln!("{}", e);
                return Box::pin(async {}.into_actor(self));
            }
            (Ok(s),) => s,
        };

        let future = self.state.send(StateActorCreateConnectionMessage::new());

        Box::pin(
            async move { future.await }
                .into_actor(self)
                .map(|result, actor, context| {
                    let id = match result {
                        Err(e) => {
                            error!("{}", e);
                            return;
                        }
                        Ok(i) => i,
                    };

                    let (reader, stream) = PacketReaderActor::new(id, context.address(), stream);
                    actor.readers.insert(id, reader);
                    actor.streams.insert(id, stream);

                    trace!("Connection {} created", id);
                }),
        )
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
        trace!("Register node");

        let (stream,): (TcpStream,) = message.into();

        let future = self.state.send(StateActorCreateConnectionMessage::new());

        (async {})
            .into_actor(self)
            .then(move |_, actor, _context| async { future.await }.into_actor(actor))
            .then(move |result, actor, context| {
                let address = context.address();

                async move {
                    let connection_id = match result {
                        Err(e) => {
                            return Err(Box::<dyn Error>::from(e));
                        }
                        Ok(c) => c,
                    };

                    let (reader, stream) = PacketReaderActor::new(connection_id, address, stream);

                    let request = match Packet::NodeRegisterRequest.to_bytes() {
                        Err(e) => {
                            return Err(Box::<dyn Error>::from(e));
                        }
                        Ok(r) => r,
                    };

                    if let Err(e) = stream.send(TcpStreamActorSendMessage::new(request)).await {
                        return Err(Box::<dyn Error>::from(e));
                    }

                    Ok((connection_id, reader, stream))
                }
                .into_actor(actor)
            })
            .map(|result, actors, context| {
                let (connection_id, reader, stream) = match result {
                    Err(e) => {
                        error!("{}", e);
                        context.stop();
                        return;
                    }
                    Ok((c, r, s)) => (c, r, s),
                };

                assert!(actors.readers.insert(connection_id, reader).is_none());
                assert!(actors.streams.insert(connection_id, stream).is_none());
            })
            .boxed_local()
    }
}

impl Handler<ConsensusActorSendBeginRequestMessage<NodeMessage>> for NodeActor {
    type Result = ResponseActFuture<
        Self,
        <ConsensusActorSendBeginRequestMessage<NodeMessage> as Message>::Result,
    >;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorSendBeginRequestMessage<NodeMessage>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        async {}
            .into_actor(self)
            .then(|_result, actor, context| {
                let state = actor.state.clone();
                let address = context.address();

                async move {
                    let nodes = match state

                        .send(StateActorReadDataMessage::new(|state| {
                            state.nodes().values().map(|x| x.clone()).collect::<Vec<_>>()
                        }))
                        .await
                    {
                        Err(e) => return Err(Box::<dyn Error>::from(e)),
                        Ok(n) => n,
                    };

                    for node in nodes.iter().filter(|n| n.connection_id().is_none()) {
                        let stream = match TcpStream::connect(node.address()).await {
                            Err(e) => {
                                error!("{}", e);
                                continue;
                            }
                            Ok(s) => s,
                        };

                        let connection_id = match state.send(StateActorCreateNodeConnectionMessage::new(node.id())).await {
                            Err(e) => return Err(Box::<dyn Error>::from(e)),
                            Ok(c) => c,
                        };

                        let (reader, stream) = PacketReaderActor::new(connection_id, address.clone(), stream);

                        //assert!(actors.readers.insert(connection_id, reader).is_none());
                        //assert!(actors.streams.insert(connection_id, stream).is_none());
                    }

                    Ok(())
                }
                .into_actor(actor)
            })
            .map(|result, _actor, context| {
                if let Err(e) = result {
                    error!("{}", e);
                    context.stop();
                }
            })
            .boxed_local()
    }
}

impl Handler<ConsensusActorSendBeginResponseMessage<NodeMessage>> for NodeActor {
    type Result = <ConsensusActorSendBeginResponseMessage<NodeMessage> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorSendBeginResponseMessage<NodeMessage>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl Handler<ConsensusActorSendCommitRequestMessage<NodeMessage>> for NodeActor {
    type Result = <ConsensusActorSendCommitRequestMessage<NodeMessage> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorSendCommitRequestMessage<NodeMessage>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl Handler<ConsensusActorSendCommitResponseMessage<NodeMessage>> for NodeActor {
    type Result = <ConsensusActorSendCommitResponseMessage<NodeMessage> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorSendCommitResponseMessage<NodeMessage>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
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

        let actor = Self::create(move |context| Self {
            consensus: ConsensusActor::new(context.address()),
            listener: TcpListenerActor::new(
                context.address().recipient(),
                Some(context.address().recipient()),
                listener,
            ),
            state: StateActor::new(),
            streams: HashMap::default(),
            readers: HashMap::default(),
        });

        match stream {
            Some(s) => {
                actor
                    .send(NodeActorRegisterMessage::new(s))
                    .await
                    .expect("Cannot send NodeActorRegisterMessage");
            }
            None => {}
        };

        Ok(actor)
    }
}
