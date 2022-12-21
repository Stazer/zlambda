use crate::follower::client::FollowerClientBuilder;
use crate::follower::FollowerHandle;
use std::error::Error;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientToNodeMessage, FollowerToGuestMessage, GuestToNodeMessage, Message, MessageStreamReader,
    MessageStreamWriter,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerConnectionBuilder {}

impl FollowerConnectionBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn task(
        self,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        follower_handle: FollowerHandle,
    ) -> FollowerConnectionTask {
        FollowerConnectionTask::new(reader, writer, follower_handle)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerConnectionTask {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    follower_handle: FollowerHandle,
}

impl FollowerConnectionTask {
    fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        follower_handle: FollowerHandle,
    ) -> Self {
        Self {
            reader,
            writer,
            follower_handle,
        }
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
    }

    pub async fn run(mut self) {
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
                        None => continue,
                        Some(Message::GuestToNode(message)) => {
                            self.on_unregistered_follower_to_node_message(message).await;
                        }
                        Some(Message::ClientToNode(message)) => {
                            if let Err(error) = self.register_client(message).await {
                                error!("{}", error);
                                break
                            }

                            break
                        },
                        Some(message) => {
                            error!("Unhandled message {:?}", message);
                            break
                        }
                    };
                }
            )
        }
    }

    async fn on_unregistered_follower_to_node_message(&mut self, message: GuestToNodeMessage) {
        match message {
            GuestToNodeMessage::RegisterRequest { .. } => {
                let leader_address = self.follower_handle.leader_address().await;

                let message =
                    Message::FollowerToGuest(FollowerToGuestMessage::RegisterNotALeaderResponse {
                        leader_address,
                    });

                let result = self.writer.write(message).await;

                if let Err(error) = result {
                    error!("{}", error);
                }
            }
            GuestToNodeMessage::HandshakeRequest { .. } => {
                let leader_address = self.follower_handle.leader_address().await;

                let message =
                    Message::FollowerToGuest(FollowerToGuestMessage::HandshakeNotALeaderResponse {
                        leader_address,
                    });

                let result = self.writer.write(message).await;

                if let Err(error) = result {
                    error!("{}", error);
                }
            }
        }
    }

    async fn register_client(
        self,
        initial_message: ClientToNodeMessage,
    ) -> Result<(), Box<dyn Error>> {
        FollowerClientBuilder::new()
            .task(
                self.reader.into(),
                self.writer.into(),
                self.follower_handle,
                initial_message,
            )
            .await
            .spawn();

        Ok(())
    }
}
