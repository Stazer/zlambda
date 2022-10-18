use crate::cluster::LeaderNodeFollowerActor;
use crate::cluster::{ConnectionId, Packet, PacketReader};
use crate::common::{ActorStopMessage, TcpStreamActor, TcpStreamActorReceiveMessage};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::mem::take;

use tokio::net::TcpStream;
use tracing::error;

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
                Packet::NodeRegisterRequest { local_address: _ } => {
                    LeaderNodeFollowerActor::new(
                        0,
                        self.id,
                        self.tcp_stream_actor_address.clone(),
                        take(&mut self.packet_reader),
                    );
                }
                _ => {
                    unimplemented!()
                }
            }
        }
    }
}

impl LeaderNodeConnectionActor {
    fn new(id: ConnectionId, tcp_stream: TcpStream) -> Addr<Self> {
        Self::create(move |context| Self {
            id,
            tcp_stream_actor_address: TcpStreamActor::new(
                context.address().recipient(),
                None,
                tcp_stream,
            ),
            packet_reader: PacketReader::default(),
        })
    }
}
