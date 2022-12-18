use crate::leader::LeaderMessage;
use std::error::Error;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tracing::error;
use zlambda_common::log::{LogEntryData, LogEntryId};
use zlambda_common::message::{
    FollowerToLeaderMessage, FollowerToLeaderMessageStreamReader, LeaderToFollowerMessage,
    LeaderToFollowerMessageStreamWriter,
};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

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
        reader: FollowerToLeaderMessageStreamReader,
        writer: LeaderToFollowerMessageStreamWriter,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollower {
    id: NodeId,
    receiver: mpsc::Receiver<LeaderFollowerMessage>,
    reader: Option<FollowerToLeaderMessageStreamReader>,
    writer: Option<LeaderToFollowerMessageStreamWriter>,
    writer_message_buffer: Vec<LeaderToFollowerMessage>,
    leader_sender: mpsc::Sender<LeaderMessage>,
}

impl LeaderFollower {
    pub fn new(
        id: NodeId,
        receiver: mpsc::Receiver<LeaderFollowerMessage>,
        reader: FollowerToLeaderMessageStreamReader,
        writer: LeaderToFollowerMessageStreamWriter,
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

    pub async fn run(mut self) {
        loop {
            self.select().await;
        }
    }

    async fn select(&mut self) {
        select!(
            read_result = { self.reader.as_mut().unwrap().read() }, if self.reader.is_some() => {
                let message = match read_result {
                    Ok(None) => {
                        self.reader = None;
                        self.writer = None;

                        return;
                    }
                    Ok(Some(message)) => message,
                    Err(error) => {
                        error!("{}", error);

                        return;
                    }
                };

                self.on_registered_follower_to_leader_message(message).await;
            }
            receive_result = self.receiver.recv() => {
                let message = match receive_result {
                    None => {
                        return
                    }
                    Some(message) => message,
                };

                self.on_message(message).await;
            }
        )
    }

    async fn on_message(&mut self, message: LeaderFollowerMessage) {
        match message {
            LeaderFollowerMessage::Replicate(
                term,
                last_committed_log_entry_id,
                log_entry_data,
                sender,
            ) => self
                .replicate(term, last_committed_log_entry_id, log_entry_data, sender)
                .await
                .expect(""),
            LeaderFollowerMessage::Handshake { reader, writer } => {
                self.on_handshake(reader, writer).await;
            }
        }
    }

    async fn on_registered_follower_to_leader_message(&mut self, message: FollowerToLeaderMessage) {
        match message {
            FollowerToLeaderMessage::AppendEntriesResponse { log_entry_ids } => {
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

    async fn on_handshake(
        &mut self,
        reader: FollowerToLeaderMessageStreamReader,
        writer: LeaderToFollowerMessageStreamWriter,
    ) {
        self.reader = Some(reader);
        self.writer = Some(writer);
    }

    async fn replicate(
        &mut self,
        term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
        sender: oneshot::Sender<()>,
    ) -> Result<(), Box<dyn Error>> {
        let message = LeaderToFollowerMessage::AppendEntriesRequest {
            term,
            last_committed_log_entry_id,
            log_entry_data,
        };

        if let Some(ref mut writer) = &mut self.writer {
            writer.write(message.clone()).await?;
        } else {
            self.writer_message_buffer.push(message);
        }

        sender
            .send(())
            .map_err(|_| Box::<dyn Error>::from("Cannot send message"))?;

        Ok(())
    }
}
