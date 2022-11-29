use crate::follower::client::FollowerClient;
use crate::follower::FollowerMessage;
use std::error::Error;
use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientMessage, ClusterMessage, ClusterMessageRegisterResponse, Message, MessageStreamReader,
    MessageStreamWriter,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerConnection {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_node_sender: mpsc::Sender<FollowerMessage>,
}

impl FollowerConnection {
    fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_node_sender: mpsc::Sender<FollowerMessage>,
    ) -> Self {
        Self {
            reader,
            writer,
            leader_node_sender,
        }
    }

    pub fn spawn(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_node_sender: mpsc::Sender<FollowerMessage>,
    ) {
        spawn(async move {
            Self::new(reader, writer, leader_node_sender).main().await;
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
                            /*if let Err(error) = self.register_follower(address).await {
                                error!("{}", error);
                            }

                            break*/
                            break
                        },
                        Some(Message::Client(ClientMessage::RegisterRequest)) => {
                            if let Err(error) = self.register_client().await {
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

    async fn register_client(mut self) -> Result<(), Box<dyn Error>> {
        self.writer
            .write(&Message::Client(ClientMessage::RegisterResponse))
            .await?;

        spawn(async move {
            FollowerClient::new(self.reader, self.writer, self.leader_node_sender)
                .run()
                .await;
        });

        Ok(())
    }
}
