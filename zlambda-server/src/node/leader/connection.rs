use crate::node::leader::LeaderNode;
use crate::node::leader::follower::LeaderNodeFollower;
use crate::node::message::{
    ClusterMessage, ClusterMessageRegisterResponse, Message, MessageStreamReader,
    MessageStreamWriter,
};
use crate::node::Node;
use crate::read_write::ReadWriteSender;
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeConnection {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    node_read_sender: ReadWriteSender<Node>,
    leader_node_read_sender: ReadWriteSender<LeaderNode>,
}

impl LeaderNodeConnection {
    fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        node_read_sender: ReadWriteSender<Node>,
        leader_node_read_sender: ReadWriteSender<LeaderNode>,
    ) -> Self {
        Self {
            reader,
            writer,
            node_read_sender,
            leader_node_read_sender,
        }
    }

    pub fn spawn(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        node_read_sender: ReadWriteSender<Node>,
        leader_node_read_sender: ReadWriteSender<LeaderNode>,
    ) {
        spawn(async move {
            Self::new(reader, writer, node_read_sender, leader_node_read_sender).main().await;
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


                            let function = move |node: &mut Node| {
                                (node.register_follower(address), node.leader_id, node.term, node.addresses.clone())
                            };

                            let (id, leader_id, term, addresses) = match self.node_read_sender.send(function).await {
                                Ok(values) => values,
                                Err(error) => {
                                    error!("{}", error);
                                    break
                                }
                            };

                            self.writer.write(&Message::Cluster(ClusterMessage::RegisterResponse(
                                ClusterMessageRegisterResponse::Ok {
                                    id, leader_id, term, addresses
                                }
                            ))).await;

                            LeaderNodeFollower::spawn(
                                id,
                                self.reader,
                                self.writer,
                                self.node_read_sender,
                                self.leader_node_read_sender,
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
