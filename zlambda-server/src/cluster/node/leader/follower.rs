use crate::cluster::{
    ConnectionId, CreateFollowerActorMessage, LeaderNodeActor, NodeId,
    NodeRegisterResponsePacketSuccessData, Packet, PacketReader,
};
use crate::common::{
    StopActorMessage, TcpStreamActor, TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
    UpdateRecipientActorMessage,
};
use actix::{Actor, Addr, AsyncContext, Context, Handler, MailboxError, Message};
use futures::{FutureExt, TryFutureExt};
use std::net::SocketAddr;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeFollowerActor {
    leader_node_actor_address: Addr<LeaderNodeActor>,
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    node_id: NodeId,
    packet_reader: PacketReader,
}

impl Actor for LeaderNodeFollowerActor {
    type Context = Context<Self>;

    #[tracing::instrument]
    fn stopped(&mut self, context: &mut Self::Context) {
    }
}

impl Handler<TcpStreamActorReceiveMessage> for LeaderNodeFollowerActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        _context: &mut <Self as Actor>::Context,
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
                _ => {
                    unimplemented!()
                }
            }
        }
    }
}

impl LeaderNodeFollowerActor {
    pub async fn new(
        leader_node_actor_address: Addr<LeaderNodeActor>,
        tcp_stream_actor_address: Addr<TcpStreamActor>,
        packet_reader: PacketReader,
        follower_socket_address: SocketAddr,
    ) -> Option<Addr<Self>> {
        let context: <Self as Actor>::Context = Context::new();

        match tcp_stream_actor_address
            .send(UpdateRecipientActorMessage::new(
                context.address().recipient(),
            ))
            .await
        {
            Ok(()) => {}
            Err(error) => {
                tcp_stream_actor_address
                    .send(StopActorMessage)
                    .map(|_result| ())
                    .await;

                Err::<(), _>(error).expect("Cannot send UpdateRecipientActorMessage");

                return None;
            }
        }

        let (node_id, leader_node_id, term_id, node_socket_addresses) =
            match leader_node_actor_address
                .send(CreateFollowerActorMessage::new(
                    follower_socket_address,
                    context.address(),
                ))
                .await
            {
                Ok(result) => result.into(),
                Err(error) => {
                    tcp_stream_actor_address
                        .send(StopActorMessage)
                        .map(|_result| ())
                        .await;

                    Err::<(), _>(error).expect("Cannot send FollowerUpgradeActorMessage");

                return None;
                }
            };

        let bytes = match (Packet::NodeRegisterResponse {
            result: Ok(NodeRegisterResponsePacketSuccessData::new(
                node_id,
                leader_node_id,
                term_id,
                node_socket_addresses.clone(),
            )),
        }.to_bytes())
        {
            Ok(bytes) => bytes,
            Err(error) => {
                tcp_stream_actor_address
                    .send(StopActorMessage)
                    .map(|_result| ())
                    .await;

                Err::<(), _>(error).expect("Cannot write NodeRegisterResponse");

                return None;
            }
        };

        match tcp_stream_actor_address.send(TcpStreamActorSendMessage::new(bytes)).await {
            Ok(result) => {
                match result {
                    Ok(()) => {
                    }
                    Err(error) => {
                        error!("{}", error);
                    },
                }
            },
            Err(error) => {
                tcp_stream_actor_address
                    .send(StopActorMessage)
                    .map(|_result| ())
                    .await;

                Err::<(), _>(error).expect("Cannot write TcpStreamActorSendMessage");

                return None;
            },
        };

        Some(context.run(Self {
            leader_node_actor_address,
            tcp_stream_actor_address,
            node_id,
            packet_reader,
        }))
    }
}
