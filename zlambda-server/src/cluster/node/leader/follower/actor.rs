use crate::cluster::{
    AcknowledgeLogEntryActorMessage, ConnectionId, CreateFollowerActorMessage, LeaderNodeActor,
    LogEntry, NodeId, NodeRegisterResponsePacketSuccessData, Packet, PacketReader,
    ReplicateLogEntryActorMessage,
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
    fn stopped(&mut self, context: &mut Self::Context) {}
}

impl Handler<ReplicateLogEntryActorMessage> for LeaderNodeFollowerActor {
    type Result = <ReplicateLogEntryActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: ReplicateLogEntryActorMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (log_entry,) = message.into();

        let bytes = (Packet::LogEntryRequest { log_entry })
            .to_bytes()
            .expect("Writing LogEntryRequest should succeed");

        self.tcp_stream_actor_address
            .try_send(TcpStreamActorSendMessage::new(bytes))
            .expect("Sending LogEntryRequest should succeed");
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
                Packet::LogEntrySuccessResponse { log_entry_id } => {
                    self.leader_node_actor_address
                        .try_send(AcknowledgeLogEntryActorMessage::new(
                            log_entry_id,
                            self.node_id,
                        ))
                        .expect("Sending AcknowledgeLogEntryActorMessage should be successful");
                }
                packet => {
                    error!("Received unhandled packet {:?}", packet);
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
        }
        .to_bytes())
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

        match tcp_stream_actor_address
            .send(TcpStreamActorSendMessage::new(bytes))
            .await
        {
            Ok(result) => match result {
                Ok(()) => {}
                Err(error) => {
                    error!("{}", error);
                }
            },
            Err(error) => {
                tcp_stream_actor_address
                    .send(StopActorMessage)
                    .map(|_result| ())
                    .await;

                Err::<(), _>(error).expect("Cannot write TcpStreamActorSendMessage");

                return None;
            }
        };

        Some(context.run(Self {
            leader_node_actor_address,
            tcp_stream_actor_address,
            node_id,
            packet_reader,
        }))
    }
}
