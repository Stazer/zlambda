use crate::log::LogEntryData;
use crate::node::message::{ClusterMessage, Message, MessageStreamReader, MessageStreamWriter};
use crate::node::{Node, NodeId};
use crate::read_write::{ReadWriteReceiver, ReadWriteSender};
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeFollower {
    id: NodeId,
    receiver: ReadWriteReceiver<LeaderNodeFollower>,
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    node_sender: ReadWriteSender<Node>,
}

impl LeaderNodeFollower {
    fn new(
        id: NodeId,
        receiver: ReadWriteReceiver<LeaderNodeFollower>,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        node_sender: ReadWriteSender<Node>,
    ) -> Self {
        Self {
            id,
            receiver,
            reader,
            writer,
            node_sender,
        }
    }

    pub fn spawn(
        id: NodeId,
        receiver: ReadWriteReceiver<LeaderNodeFollower>,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        node_sender: ReadWriteSender<Node>,
    ) {
        spawn(async move {
            Self::new(id, receiver, reader, writer, node_sender)
                .main()
                .await;
        });
    }

    async fn main(mut self) {
        loop {
            select!(
                read_result = self.reader.read() => {
                    let message = match read_result {
                        Ok(None) => break,
                        Ok(Some(message)) => message,
                        Err(error) => {
                            error!("{}", error);
                            break
                        }
                    };

                    match message {
                        Message::Cluster(ClusterMessage::AppendEntriesResponse { log_entry_ids }) => {
                            let result = self.node_sender.send({
                                let id = self.id;

                                move |node| {
                                    node.acknowledge(log_entry_ids, id);
                                }
                            }).await;

                            if let Err(error) = result {
                                error!("{}", error);
                                break
                            }
                        }
                        message => {
                            error!("Unexpected message {:?}", message);
                            break
                        }
                    };
                }
                receive_result = self.receiver.recv() => {
                    let function = match receive_result {
                        None => {
                            break
                        }
                        Some(function) => function,
                    };

                    function(&mut self);
                }
            )
        }
    }

    pub async fn replicate(&mut self, log_entry_data: LogEntryData) {}
}
