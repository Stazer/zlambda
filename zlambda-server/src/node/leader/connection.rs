use crate::node::leader::client::LeaderNodeClient;
use crate::node::leader::follower::LeaderNodeFollower;
use crate::node::leader::LeaderNodeMessage;
use crate::node::message::{
    ClientMessage, ClusterMessage, ClusterMessageRegisterResponse, Message, MessageStreamReader,
    MessageStreamWriter,
};
use std::error::Error;
use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeConnection {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_node_sender: mpsc::Sender<LeaderNodeMessage>,
}

impl LeaderNodeConnection {
    fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_node_sender: mpsc::Sender<LeaderNodeMessage>,
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
        leader_node_sender: mpsc::Sender<LeaderNodeMessage>,
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
                            if let Err(error) = self.register_follower(address).await {
                                error!("{}", error);
                            }

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

    async fn register_follower(mut self, address: SocketAddr) -> Result<(), Box<dyn Error>> {
        let (follower_sender, follower_receiver) = mpsc::channel(16);
        let (result_sender, result_receiver) = oneshot::channel();

        self.leader_node_sender
            .send(LeaderNodeMessage::Register {
                address,
                follower_sender,
                result_sender,
            })
            .await?;

        let (id, leader_id, term, addresses) = result_receiver.await?;

        self.writer
            .write(&Message::Cluster(ClusterMessage::RegisterResponse(
                ClusterMessageRegisterResponse::Ok {
                    id,
                    leader_id,
                    term,
                    addresses,
                },
            )))
            .await?;

        spawn(async move {
            LeaderNodeFollower::new(
                id,
                follower_receiver,
                self.reader,
                self.writer,
                self.leader_node_sender,
            )
            .run()
            .await;
        });

        Ok(())
    }

    async fn register_client(mut self) -> Result<(), Box<dyn Error>> {
        self.writer
            .write(&Message::Client(ClientMessage::RegisterResponse))
            .await?;

        spawn(async move {
            LeaderNodeClient::new(self.reader, self.writer, self.leader_node_sender)
                .run()
                .await;
        });

        Ok(())
    }
}
