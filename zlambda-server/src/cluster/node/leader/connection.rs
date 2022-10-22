use crate::cluster::LeaderNodeFollowerActor;
use crate::cluster::{ConnectionId, LeaderNodeActor, Packet, PacketReader};
use crate::common::{TcpStreamActor, TcpStreamActorReceiveMessage};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message, ActorFutureExt, WrapFuture};
use std::mem::take;
use tokio::net::TcpStream;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeConnectionActor {
    id: ConnectionId,
    leader_node_actor_address: Addr<LeaderNodeActor>,
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    packet_reader: PacketReader,
}

impl Actor for LeaderNodeConnectionActor {
    type Context = Context<Self>;
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
                context.stop();

                return;
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
                    context.stop();

                    return;
                }
            };

            match packet {
                Packet::NodeRegisterRequest { local_address: _ } => {
                    let leader_node_actor_address = self.leader_node_actor_address.clone();
                    let tcp_stream_actor_address = self.tcp_stream_actor_address.clone();
                    let connection_id = self.id;
                    let packet_reader = take(&mut self.packet_reader);

                    context.wait(
                        async move {
                            LeaderNodeFollowerActor::new(
                                leader_node_actor_address,
                                tcp_stream_actor_address,
                                connection_id,
                                packet_reader,
                            ).await
                        }
                        .into_actor(self)
                            .map(|result, _actor, context| {
                                match result {
                                    Ok(_) => {},
                                    Err(e) => {
                                        error!("{}", e);
                                        context.stop();
                                    }
                                }
                            }),
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
    pub fn new(
        leader_node_actor_address: Addr<LeaderNodeActor>,
        tcp_stream: TcpStream,
        id: ConnectionId,
    ) -> Addr<Self> {
        Self::create(move |context| Self {
            id,
            leader_node_actor_address,
            tcp_stream_actor_address: TcpStreamActor::new(
                context.address().recipient(),
                tcp_stream,
            ),
            packet_reader: PacketReader::default(),
        })
    }
}
