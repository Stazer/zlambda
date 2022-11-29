use crate::leader::LeaderMessage;
use tokio::select;
use tokio::sync::mpsc;
use tracing::error;
use zlambda_common::log::LogEntryData;
use zlambda_common::message::{ClusterMessage, Message, MessageStreamReader, MessageStreamWriter};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LeaderFollowerMessage {
    Replicate {
        term: Term,
        log_entry_data: Vec<LogEntryData>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollower {
    id: NodeId,
    receiver: mpsc::Receiver<LeaderFollowerMessage>,
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_sender: mpsc::Sender<LeaderMessage>,
}

impl LeaderFollower {
    pub fn new(
        id: NodeId,
        receiver: mpsc::Receiver<LeaderFollowerMessage>,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_sender: mpsc::Sender<LeaderMessage>,
    ) -> Self {
        Self {
            id,
            receiver,
            reader,
            writer,
            leader_sender,
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
                            let result = self.leader_sender.send(LeaderMessage::Acknowledge {
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
                        LeaderFollowerMessage::Replicate { term, log_entry_data } => self.replicate(term, log_entry_data).await,
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
