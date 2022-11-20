use zlambda_common::log::LogEntryData;
use crate::leader::LeaderNodeMessage;
use zlambda_common::message::{ClusterMessage, Message, MessageStreamReader, MessageStreamWriter};
use zlambda_common::node::{NodeId};
use zlambda_common::term::{Term};
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LeaderNodeFollowerMessage {
    Replicate {
        term: Term,
        log_entry_data: Vec<LogEntryData>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeFollower {
    id: NodeId,
    receiver: mpsc::Receiver<LeaderNodeFollowerMessage>,
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_node_sender: mpsc::Sender<LeaderNodeMessage>,
}

impl LeaderNodeFollower {
    pub fn new(
        id: NodeId,
        receiver: mpsc::Receiver<LeaderNodeFollowerMessage>,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_node_sender: mpsc::Sender<LeaderNodeMessage>,
    ) -> Self {
        Self {
            id,
            receiver,
            reader,
            writer,
            leader_node_sender,
        }
    }

    pub async fn run(mut self) {
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
                            let result = self.leader_node_sender.send(LeaderNodeMessage::Acknowledge {
                                node_id: self.id,
                                log_entry_ids,
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
                    let message = match receive_result {
                        None => {
                            break
                        }
                        Some(message) => message,
                    };

                    match message {
                        LeaderNodeFollowerMessage::Replicate { term, log_entry_data } => self.replicate(term, log_entry_data).await,
                    }
                }
            )
        }
    }

    async fn replicate(&mut self, term: Term, log_entry_data: Vec<LogEntryData>) {
        let result = self
            .writer
            .write(&Message::Cluster(ClusterMessage::AppendEntriesRequest {
                term,
                log_entry_data,
            }))
            .await;

        if let Err(error) = result {
            error!("{}", error);
            return;
        }
    }
}