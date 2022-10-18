use crate::cluster::{
    ConnectionId, NodeActorRemoveConnectionMessage, NodeId, Packet, PacketReaderActor,
    PacketReaderActorReadPacketMessage, PacketReader,
};
use crate::common::{
    ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor,
    TcpStreamActorReceiveMessage,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeFollowerActor {
    connection_id: ConnectionId,
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    packet_reader: PacketReader,
}

impl Actor for LeaderNodeFollowerActor {
    type Context = Context<Self>;
}

impl Handler<TcpStreamActorReceiveMessage> for LeaderNodeFollowerActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl LeaderNodeFollowerActor {
    pub fn new(
        connection_id: ConnectionId,
        tcp_stream_actor_address: Addr<TcpStreamActor>,
        packet_reader: PacketReader
    ) -> Addr<Self> {
        (Self {
            connection_id,
            tcp_stream_actor_address,
            packet_reader,
        }).start()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeConnectionActor {
    id: ConnectionId,
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    packet_reader: PacketReader,
}

impl Actor for LeaderNodeConnectionActor {
    type Context = Context<Self>;
}

impl Handler<ActorStopMessage> for LeaderNodeConnectionActor {
    type Result = <ActorStopMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop()
    }
}

impl Handler<TcpStreamActorReceiveMessage> for LeaderNodeConnectionActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (bytes,) = message.into();

        let bytes = match bytes {
            Ok(bytes) => bytes,
            Err(error) => {
                error!("{}", error);
                todo!();
            }
        };

        self.packet_reader.push(bytes);

        loop {
            let packet = match self.packet_reader.next() {
                Ok(Some(packet)) => packet,
                Ok(None) => {
                    break;
                }
                Err(error) => {
                    error!("{}", error);
                    todo!();
                }
            };

            match packet {
                Packet::NodeRegisterRequest { local_address } => {

                }
                _ => {
                    unimplemented!()
                }
            }
        }
    }
}

impl Handler<NodeActorRemoveConnectionMessage<()>> for LeaderNodeConnectionActor {
    type Result = <NodeActorRemoveConnectionMessage<()> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage<()>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl LeaderNodeConnectionActor {
    fn new(id: ConnectionId, tcp_stream: TcpStream) -> Addr<Self> {
        Self::create(move |context| {
            Self {
                id,
                tcp_stream_actor_address: TcpStreamActor::new(context.address().recipient(), None, tcp_stream),
                packet_reader: PacketReader::default(),
            }
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeActor {
    tcp_listener_actor_address: Addr<TcpListenerActor>,
    connection_actor_addresses: HashMap<ConnectoinId, ()>,
    node_actor_addresses: HashMap<NodeId, ()>,
}

impl Actor for LeaderNodeActor {
    type Context = Context<Self>;
}

impl Handler<ActorStopMessage> for LeaderNodeActor {
    type Result = <ActorStopMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop()
    }
}

impl Handler<TcpListenerActorAcceptMessage> for LeaderNodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result,) = message.into();

        match result {
            Ok(tcp_stream) => {

            }
            Err(error) => {
                error!("{}", error);
            }
        }
    }
}

impl LeaderNodeActor {
    pub fn new(tcp_listener: TcpListener) -> Addr<LeaderNodeActor> {
        Self::create(move |context| Self {
            connection_actor_addresses: HashMap::<_, ()>::default(),
            tcp_listener_actor_address: TcpListenerActor::new(
                context.address().recipient(),
                Some(context.address().recipient()),
                tcp_listener,
            ),
            node_actor_addresses: HashMap::<_, ()>::default(),
        })
    }
}
