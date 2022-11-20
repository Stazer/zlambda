use crate::node::leader::follower::LeaderNodeFollower;
use crate::node::leader::LeaderNodeMessage;
use crate::node::message::{
    ClusterMessage, ClusterMessageRegisterResponse, Message, MessageStreamReader,
    MessageStreamWriter,
};
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
                            let (follower_sender, follower_receiver) = mpsc::channel(16);
                            let (result_sender, result_receiver) = oneshot::channel();

                            let result = self.leader_node_sender.send(LeaderNodeMessage::Register {
                                address,
                                follower_sender,
                                result_sender,
                            }).await;

                            if let Err(error) = result {
                                error!("{}", error);
                                break
                            }

                            let (id, leader_id, term, addresses) = match result_receiver.await {
                                Err(error) => {
                                    error!("{}", error);
                                    break
                                },
                                Ok(values) => values,
                            };

                            let result = self.writer.write(&Message::Cluster(ClusterMessage::RegisterResponse(
                                ClusterMessageRegisterResponse::Ok {
                                    id, leader_id, term, addresses
                                }
                            ))).await;

                            if let Err(error) = result {
                                error!("{}", error);
                                break;
                            }

                            LeaderNodeFollower::spawn(
                                id,
                                follower_receiver,
                                self.reader,
                                self.writer,
                                self.leader_node_sender,
                            );

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
}
