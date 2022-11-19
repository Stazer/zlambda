use crate::node::leader::follower::LeaderNodeFollower;
use crate::node::message::{
    ClusterMessage, ClusterMessageRegisterResponse, Message, MessageStreamReader,
    MessageStreamWriter,
};
use crate::node::Node;
use crate::read_write::{read_write_channel, ReadWriteSender};
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeConnection {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    node_sender: ReadWriteSender<Node>,
}

impl LeaderNodeConnection {
    fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        node_sender: ReadWriteSender<Node>,
    ) -> Self {
        Self {
            reader,
            writer,
            node_sender,
        }
    }

    pub fn spawn(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        node_sender: ReadWriteSender<Node>,
    ) {
        spawn(async move {
            Self::new(reader, writer, node_sender).main().await;
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
                            let (sender, receiver) = read_write_channel();

                            let function = move |node: &mut Node| {
                                (node.register_follower(address, sender), node.leader_id, node.term, node.addresses.clone())
                            };

                            let (id, leader_id, term, addresses) = match self.node_sender.send(function).await {
                                Ok(values) => values,
                                Err(error) => {
                                    error!("{}", error);
                                    break
                                }
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
                                receiver,
                                self.reader,
                                self.writer,
                                self.node_sender,
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
