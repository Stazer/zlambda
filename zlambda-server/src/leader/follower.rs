use crate::leader::LeaderMessage;
use std::error::Error;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tracing::error;
use zlambda_common::log::{LogEntryData, LogEntryId};
use zlambda_common::message::{
    LeaderToRegisteredFollowerMessage, LeaderToRegisteredFollowerMessageStreamWriter,
    RegisteredFollowerToLeaderMessage, RegisteredFollowerToLeaderMessageStreamReader,
};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;
use std::future::pending;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LeaderFollowerMessage {
    Replicate(
        Term,
        Option<LogEntryId>,
        Vec<LogEntryData>,
        oneshot::Sender<()>,
    ),
    Handshake {
        reader: RegisteredFollowerToLeaderMessageStreamReader,
        writer: LeaderToRegisteredFollowerMessageStreamWriter,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollower {
    id: NodeId,
    receiver: mpsc::Receiver<LeaderFollowerMessage>,
    reader: Option<RegisteredFollowerToLeaderMessageStreamReader>,
    writer: Option<LeaderToRegisteredFollowerMessageStreamWriter>,
    writer_message_buffer: Vec<LeaderToRegisteredFollowerMessage>,
    leader_sender: mpsc::Sender<LeaderMessage>,
}

impl LeaderFollower {
    pub fn new(
        id: NodeId,
        receiver: mpsc::Receiver<LeaderFollowerMessage>,
        reader: RegisteredFollowerToLeaderMessageStreamReader,
        writer: LeaderToRegisteredFollowerMessageStreamWriter,
        leader_sender: mpsc::Sender<LeaderMessage>,
    ) -> Self {
        Self {
            id,
            receiver,
            reader: Some(reader),
            writer: Some(writer),
            writer_message_buffer: Vec::default(),
            leader_sender,

        }
    }

    pub async fn run(mut self)
    {
        let mut reader = self.reader;

        let a = async move {
            if let Some(ref mut reader) = &mut std::mem::take(&mut reader) {
                    reader.read().await
            } else {
                pending().await
            }
        };

        loop {
            use futures::future::{OptionFuture, TryFuture, TryFutureExt};

            //let a = OptionFuture::from(self.reader.map(|x| x.read())).unwrap_or_else(|| pending());

            select!(
                read_result = a => {
                    let message = match read_result {
                        Ok(None) => {
                            self.reader = None;
                            self.writer = None;

                            continue
                        }
                        Ok(Some(message)) => message,
                        Err(error) => {
                            error!("{}", error);

                            break
                        }
                    };

                    self.on_registered_follower_to_leader_message(message).await;
                }
                receive_result = self.receiver.recv() => {
                    let message = match receive_result {
                        None => {
                            error!("Receiver closed");
                            break
                        }
                        Some(message) => message,
                    };

                }
            )
        }
    }

    async fn on_message(
        &mut self,
        message: LeaderFollowerMessage,
    ) {
        match message {
            LeaderFollowerMessage::Replicate(term, last_committed_log_entry_id, log_entry_data, sender) => self.replicate(
                term,
                last_committed_log_entry_id,
                log_entry_data,
                sender,
            ).await.expect(""),
            LeaderFollowerMessage::Handshake {
                reader,
                writer
            } => {

            }
        }
    }

    async fn on_registered_follower_to_leader_message(
        &mut self,
        message: RegisteredFollowerToLeaderMessage,
    ) {
        match message {
            RegisteredFollowerToLeaderMessage::AppendEntriesResponse { log_entry_ids } => {
                let result = self
                    .leader_sender
                    .send(LeaderMessage::Acknowledge(log_entry_ids, self.id))
                    .await;

                if let Err(error) = result {
                    error!("{}", error);
                }
            }
        }
    }

    async fn replicate(
        &mut self,
        term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
        sender: oneshot::Sender<()>,
    ) -> Result<(), Box<dyn Error>> {
        let message = LeaderToRegisteredFollowerMessage::AppendEntriesRequest {
            term,
            last_committed_log_entry_id,
            log_entry_data,
        };

        if let Some(ref mut writer) = &mut self.writer {
            writer
                .write(message.clone())
                .await?;
        } else {
            self.writer_message_buffer.push(message);
        }

        sender
            .send(())
            .map_err(|_| Box::<dyn Error>::from("Cannot send message"))?;

        Ok(())
    }
}
