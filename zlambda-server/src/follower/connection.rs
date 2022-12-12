use crate::follower::client::FollowerClient;
use crate::follower::FollowerMessage;
use std::error::Error;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientToNodeMessage, ClusterMessage, ClusterMessageRegisterResponse, Message,
    MessageStreamReader, MessageStreamWriter,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerConnection {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    follower_sender: mpsc::Sender<FollowerMessage>,
}

impl FollowerConnection {
    fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        follower_sender: mpsc::Sender<FollowerMessage>,
    ) -> Self {
        Self {
            reader,
            writer,
            follower_sender,
        }
    }

    pub fn spawn(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        follower_sender: mpsc::Sender<FollowerMessage>,
    ) {
        spawn(async move {
            Self::new(reader, writer, follower_sender).main().await;
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
                        None => continue,
                        Some(Message::Cluster(ClusterMessage::RegisterRequest { address })) => {
                            let (sender, receiver) = oneshot::channel();

                            self.follower_sender.send(FollowerMessage::ReadLeaderAddress {
                                sender,
                            }).await.expect("Cannot send FollowerMessage::ReadLeaderAddress");

                            let leader_address = match receiver.await {
                                Err(error) => {
                                    error!("{}", error);
                                    break
                                }
                                Ok(leader_address) => leader_address,
                            };

                            let message = Message::Cluster(
                                ClusterMessage::RegisterResponse(
                                    ClusterMessageRegisterResponse::NotALeader {
                                        leader_address
                                    },
                                ),
                            );

                            let result = self.writer.write(message).await;

                            if let Err(error) = result {
                                error!("{}", error);
                                break
                            }

                            break
                        },
                        Some(Message::ClientToNode(message)) => {
                            if let Err(error) = self.register_client(message).await {
                                error!("{}", error);
                                break
                            }

                            break
                        },
                        /*Some(Message::Client(ClientMessage::RegisterRequest)) => {
                            if let Err(error) = self.register_client().await {
                                error!("{}", error);
                                break
                            }

                            break
                        }*/
                        Some(message) => {
                            error!("Unhandled message {:?}", message);
                            break
                        }
                    };
                }
            )
        }
    }

    async fn register_client(
        mut self,
        initial_message: ClientToNodeMessage,
    ) -> Result<(), Box<dyn Error>> {
        spawn(async move {
            FollowerClient::new(
                self.reader.into(),
                self.writer.into(),
                self.follower_sender,
                initial_message,
            )
            .await
            .run()
            .await;
        });

        Ok(())
    }
}
