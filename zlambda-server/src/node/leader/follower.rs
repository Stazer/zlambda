use crate::node::leader::LeaderNode;
use crate::node::{Node, NodeId};
use crate::read_write::ReadWriteSender;
use crate::node::message::{
    MessageStreamReader,
    MessageStreamWriter,
};
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeFollower {
    id: NodeId,
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    node_read_sender: ReadWriteSender<Node>,
    leader_node_read_sender: ReadWriteSender<LeaderNode>,
}

impl LeaderNodeFollower {
    fn new(id: NodeId, reader: MessageStreamReader, writer: MessageStreamWriter,
        node_read_sender: ReadWriteSender<Node>,
        leader_node_read_sender: ReadWriteSender<LeaderNode>,
    ) -> Self {
        Self { id, reader, writer,
            node_read_sender,
            leader_node_read_sender,
        }
    }

    pub fn spawn(id: NodeId, reader: MessageStreamReader, writer: MessageStreamWriter,
        node_read_sender: ReadWriteSender<Node>,
        leader_node_read_sender: ReadWriteSender<LeaderNode>,
    ) {
        spawn(async move {
            Self::new(id, reader, writer, node_read_sender, leader_node_read_sender).main().await;
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
