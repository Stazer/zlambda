use crate::cluster::LeaderNodeFollowerActor;
use crate::cluster::{LeaderNodeActor, Packet, PacketReader};
use crate::common::{StopActorMessage, TcpStreamActor, TcpStreamActorReceiveMessage};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, WrapFuture,
};
use std::mem::take;
use tokio::net::TcpStream;
use tracing::{error};
use futures::FutureExt;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeConnectionActor {
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
        let (result,) = message.into();

        let bytes = match result {
            Ok(bytes) => bytes,
            Err(error) => {
                error!("{}", error);

                let future = self.tcp_stream_actor_address.send(StopActorMessage);

                context.wait(
                    async move {
                        future.await.expect("Cannot send StopActorMessage");
                    }
                    .into_actor(self)
                    .map(|_result, _actor, context| {
                        context.stop();
                    }),
                );

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

                    let future = self.tcp_stream_actor_address.send(StopActorMessage);

                    context.wait(
                        async move {
                            future.await.expect("Cannot send StopActorMessage");
                        }
                        .into_actor(self)
                        .map(|_result, _actor, context| {
                            context.stop();
                        }),
                    );

                    return;
                }
            };

            match packet {
                Packet::NodeRegisterRequest { local_address } => {
                    let future = LeaderNodeFollowerActor::new(
                        self.leader_node_actor_address.clone(),
                        self.tcp_stream_actor_address.clone(),
                        take(&mut self.packet_reader),
                        local_address,
                    ).map(|_address| {});

                    context.wait(async move { future.await }.into_actor(self));
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
    ) -> Addr<Self> {
        Self::create(move |context| Self {
            leader_node_actor_address,
            tcp_stream_actor_address: TcpStreamActor::new(
                context.address().recipient(),
                tcp_stream,
            ),
            packet_reader: PacketReader::default(),
        })
    }
}
