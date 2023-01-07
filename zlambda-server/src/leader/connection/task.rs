use crate::leader::client::LeaderClientBuilder;
use crate::leader::follower::LeaderFollowerBuilder;
use crate::leader::LeaderHandle;
use std::error::Error;
use std::net::SocketAddr;
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientToNodeMessage, GuestToNodeMessage, LeaderToGuestMessage, Message, MessageStreamReader,
    MessageStreamWriter,
};
use zlambda_common::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderConnectionTask {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_handle: LeaderHandle,
}

impl LeaderConnectionTask {
    pub fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_handle: LeaderHandle,
    ) -> Self {
        Self {
            reader,
            writer,
            leader_handle,
        }
    }

    pub fn spawn(self) {
        spawn(async move {
            self.main().await;
        });
    }

    async fn main(mut self) {
        loop {
            select!(
                read_result = self.reader.read() => {
                    let message = match read_result {
                        Ok(message) => message,
                        Err(error) => {
                            error!("{}", error);
                            break
                        }
                    };

                    match message {
                        None => {
                            break
                        },
                        Some(Message::GuestToNode(message)) => {
                            self.on_unregistered_to_node_message(message).await;
                            break
                        },
                        Some(Message::ClientToNode(message)) => {
                            if let Err(error) = self.register_client(message).await {
                                error!("{}", error);
                            }

                            break
                        }
                        Some(message) => {
                            error!("Unhandled message {:?}", message);
                            break
                        }
                    };
                }
            )
        }
    }

    async fn on_unregistered_to_node_message(self, message: GuestToNodeMessage) {
        match message {
            GuestToNodeMessage::RegisterRequest { address } => {
                self.register_follower(address).await.expect("");
            }
            GuestToNodeMessage::HandshakeRequest { address, node_id } => {
                self.handshake_follower(address, node_id).await.expect("");
            }
        }
    }

    async fn register_follower(self, address: SocketAddr) -> Result<(), Box<dyn Error>> {
        let builder = LeaderFollowerBuilder::new();

        let (id, leader_id, term, addresses) =
            self.leader_handle.register(address, builder.handle()).await;

        let mut writer = self.writer.into();

        writer
            .write(LeaderToGuestMessage::RegisterOkResponse {
                id,
                leader_id,
                term,
                addresses,
            })
            .await?;

        builder
            .build(
                id,
                Some(self.reader.into()),
                Some(writer.into()),
                self.leader_handle,
            )
            .spawn();

        Ok(())
    }

    async fn handshake_follower(
        self,
        address: SocketAddr,
        node_id: NodeId,
    ) -> Result<(), Box<dyn Error>> {
        match self.leader_handle.handshake(node_id, address).await {
            Ok((
                term,
                acknowledging_log_entry_data,
                last_committed_log_entry_id,
                follower_handle,
            )) => {
                follower_handle
                    .handshake(
                        self.reader.into(),
                        self.writer.into(),
                        term,
                        acknowledging_log_entry_data,
                        last_committed_log_entry_id,
                    )
                    .await
                    .expect("");
            }
            Err(message) => {
                let mut writer = self.writer.into();

                writer
                    .write(LeaderToGuestMessage::HandshakeErrorResponse { message })
                    .await
                    .expect("");
            }
        };

        Ok(())
    }

    async fn register_client(
        self,
        initial_message: ClientToNodeMessage,
    ) -> Result<(), Box<dyn Error>> {
        LeaderClientBuilder::new()
            .task(
                self.reader.into(),
                self.writer.into(),
                self.leader_handle,
                initial_message,
            )
            .await
            .spawn();

        Ok(())
    }
}
