use crate::leader::LeaderMessage;
use std::error::Error;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tracing::error;
use zlambda_common::log::LogEntryData;
use zlambda_common::message::{ClusterMessage, Message, MessageStreamReader, MessageStreamWriter};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LeaderFollowerMessage {
    Replicate(Term, Vec<LogEntryData>, oneshot::Sender<()>),
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
                        Ok(None) => {
                            todo!("Follower crashed");
                            break
                        }
                        Ok(Some(message)) => message,
                        Err(error) => {
                            error!("{}", error);
                            break
                        }
                    };

                    match message {
                        Message::Cluster(ClusterMessage::AppendEntriesResponse { log_entry_ids }) => {
                            let result = self.leader_sender.send(LeaderMessage::Acknowledge(
                                log_entry_ids,
                                self.id,
                            )).await;

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
                            error!("Receiver closed");
                            break
                        }
                        Some(message) => message,
                    };

                    match message {
                        LeaderFollowerMessage::Replicate(term, log_entry_data, sender) => self.replicate(
                            term,
                            log_entry_data,
                            sender,
                        ).await.expect(""),
                    }
                }
            )
        }
    }

    async fn replicate(
        &mut self,
        term: Term,
        log_entry_data: Vec<LogEntryData>,
        sender: oneshot::Sender<()>,
    ) -> Result<(), Box<dyn Error>> {
        self.writer
            .write(&Message::Cluster(ClusterMessage::AppendEntriesRequest {
                term,
                log_entry_data,
            }))
            .await?;

        sender
            .send(())
            .map_err(|_| Box::<dyn Error>::from("Cannot send message"))?;

        Ok(())
    }
}
