use crate::algorithm::next_key;
use crate::cluster::{
    ConsensusActor,
    CreateConnectionNodeStateTransition,
    DestroyConnectionNodeStateTransition, NodeClientId, NodeConnectionId, NodeId, NodeStateData,
    NodeStateTransition, Packet, PacketReaderActor, PacketReaderActorReadPacketMessage,
    ReadPacketError,
    RegisterNodeNodeStateTransition,
    ConsensusActorSendBeginRequestMessage,
    ConsensusActorSendBeginResponseMessage,
    ConsensusActorSendCommitRequestMessage,
    ConsensusActorSendCommitResponseMessage,
    NodeActorRemoveConnectionMessage,
};
use crate::common::{
    ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor,
    TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use async_trait::async_trait;
use std::collections::HashMap;
use std::io;
use std::rc::Rc;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};
use tracing_subscriber::fmt::init;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type CommandType = ();

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActor {
    consensus: Addr<ConsensusActor<CommandType, Self>>,
    state: NodeStateData,
    listener: Addr<TcpListenerActor>,
    streams: HashMap<NodeConnectionId, Addr<TcpStreamActor>>,
    readers: HashMap<NodeConnectionId, Addr<PacketReaderActor>>,
}

impl Actor for NodeActor {
    type Context = Context<Self>;

    #[tracing::instrument]
    fn stopped(&mut self, _context: &mut Self::Context) {
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
    type Result = <PacketReaderActorReadPacketMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        /*trace!("Received packet");
        let (id, packet) = message.into();

        let client = match self.clients.get_mut(&id) {
            Some(c) => c,
            None => {
                panic!("PacketReaderActorReadPacketMessage should never be received by NodeActor with an unknown id {:?}", id);
            }
        };

        match (client.r#type().as_ref(), packet) {
            (None, Packet::FollowerHandshakeChallenge) => {
                let id = next_key(&mut self.followers);

                let follower = Rc::new(NodeFollower::new(id));
                if self.followers.insert(id, follower.clone()).is_some() {
                    panic!("followers::insert with id {} should never return Some", id);
                }
                client.set_type(Some(NodeClientType::Follower(follower)));

                let packet = match (Packet::FollowerHandshakeSuccess { id }).to_vec() {
                    Err(e) => {
                        eprintln!("{}", e);
                        context.stop();
                        return;
                    }
                    Ok(p) => p,
                };

                client
                    .stream()
                    .do_send(TcpStreamActorSendMessage::new(packet.into()));
            }
            _ => {
                error!("Unhandled packet");
            }
        };*/
    }
}

impl Handler<NodeActorRemoveConnectionMessage> for NodeActor {
    type Result = <NodeActorRemoveConnectionMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (id,) = message.into();

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

        DestroyConnectionNodeStateTransition::new(id).apply(&mut self.state);

        trace!("Connection destroyed");
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
        let stream = match message.into() {
            (Err(e),) => {
                eprintln!("{}", e);
                return;
            }
            (Ok(s),) => s,
        };

        let id = CreateConnectionNodeStateTransition.apply(&mut self.state);
        let (reader, stream) = PacketReaderActor::new(id, context.address(), stream);
        self.readers.insert(id, reader);
        self.streams.insert(id, stream);

        trace!("Connnection created");
    }
}

impl Handler<ConsensusActorSendBeginRequestMessage<CommandType>> for NodeActor {
    type Result = <ConsensusActorSendBeginRequestMessage<CommandType> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorSendBeginRequestMessage<CommandType>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl Handler<ConsensusActorSendBeginResponseMessage<CommandType>> for NodeActor {
    type Result = <ConsensusActorSendBeginResponseMessage<CommandType> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorSendBeginResponseMessage<CommandType>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl Handler<ConsensusActorSendCommitRequestMessage<CommandType>> for NodeActor {
    type Result = <ConsensusActorSendCommitRequestMessage<CommandType> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorSendCommitRequestMessage<CommandType>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl Handler<ConsensusActorSendCommitResponseMessage<CommandType>> for NodeActor {
    type Result = <ConsensusActorSendCommitResponseMessage<CommandType> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ConsensusActorSendCommitResponseMessage<CommandType>,
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
        init();

        let listener = TcpListener::bind(listener_address).await?;

        match stream_address {
            Some(stream_address) => {
                let stream = TcpStream::connect(stream_address).await?;

                Ok(Self::create(move |context| {
                    let mut state = NodeStateData::default();
                    let id = RegisterNodeNodeStateTransition.apply(&mut state);

                    let mut readers = HashMap::default();
                    let mut streams = HashMap::default();

                    let (reader, stream) = PacketReaderActor::new(id, context.address(), stream);
                    readers.insert(id, reader);
                    streams.insert(id, stream);

                    Self {
                        consensus: ConsensusActor::new(context.address()),
                        listener: TcpListenerActor::new(
                            context.address().recipient(),
                            Some(context.address().recipient()),
                            listener,
                        ),
                        state,
                        streams,
                        readers,
                    }
                }))
            }
            None => Ok(Self::create(move |context| Self {
                consensus: ConsensusActor::new(context.address()),
                listener: TcpListenerActor::new(
                    context.address().recipient(),
                    Some(context.address().recipient()),
                    listener,
                ),
                state: NodeStateData::default(),
                streams: HashMap::default(),
                readers: HashMap::default(),
            })),
        }
    }
}
