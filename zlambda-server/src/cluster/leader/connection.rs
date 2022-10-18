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
