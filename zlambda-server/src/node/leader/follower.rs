use crate::node::leader::LeaderNode;
use crate::node::Node;
use crate::read_write::ReadWriteSender;
use crate::node::message::{
    ClusterMessage, ClusterMessageRegisterResponse, Message, MessageStreamReader,
    MessageStreamWriter,
};
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeFollower {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    node_read_sender: ReadWriteSender<Node>,
    leader_node_read_sender: ReadWriteSender<LeaderNode>,
}

impl LeaderNodeFollower {
    fn new(reader: MessageStreamReader, writer: MessageStreamWriter,
        node_read_sender: ReadWriteSender<Node>,
        leader_node_read_sender: ReadWriteSender<LeaderNode>,
    ) -> Self {
        Self { reader, writer,
            node_read_sender,
            leader_node_read_sender,
        }
    }

    pub fn spawn(reader: MessageStreamReader, writer: MessageStreamWriter,
        node_read_sender: ReadWriteSender<Node>,
        leader_node_read_sender: ReadWriteSender<LeaderNode>,
    ) {
        spawn(async move {
            let mut follower =
                Self::new(reader, writer, node_read_sender, leader_node_read_sender);

            follower.main().await;
        });
    }

    async fn main(&mut self) {
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
