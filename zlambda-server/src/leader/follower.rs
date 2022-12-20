use crate::leader::LeaderMessage;
use std::error::Error;
use tokio::{select, spawn};
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

#[derive(Clone, Debug)]
pub struct LeaderFollowerHandle {
    sender: mpsc::Sender<LeaderFollowerMessage>,
}

impl LeaderFollowerHandle {
    pub fn new(
        sender: mpsc::Sender<LeaderFollowerMessage>,
    ) -> Self {
        Self {
            sender,
        }
    }

    pub async fn status(&self) -> LeaderFollowerStatus {
        let (sender, receiver) = oneshot::channel();

        self.sender.send(LeaderFollowerMessage::Status {
            sender,
        }).await.expect("LeaderFollowerMessage::Status should be sent");

        receiver.await.expect("Value should be received")
    }

    pub async fn replicate(
        &self,
        term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
    ) {
        let (sender, receiver) = oneshot::channel();

        self.sender.send(LeaderFollowerMessage::Replicate (
            term,
            last_committed_log_entry_id,
            log_entry_data,
            sender,
        )).await.expect("LeaderFollowerMessage::Replicate should be sent");

        receiver.await.expect("Value should be received")
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollowerStatus {
    connected: bool
}

impl LeaderFollowerStatus {
    pub fn new(
        connected: bool
    ) -> Self {
        Self {
            connected,
        }
    }

    pub fn connected(&self) -> bool {
        self.connected
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LeaderFollowerMessage {
    Replicate(
        Term,
        Option<LogEntryId>,
        Vec<LogEntryData>,
        oneshot::Sender<()>,
    ),
    Status {
        sender: oneshot::Sender<LeaderFollowerStatus>,
    },
    Handshake {
        reader: FollowerToLeaderMessageStreamReader,
        writer: LeaderToFollowerMessageStreamWriter,
        result_sender: oneshot::Sender<Result<(), String>>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollowerTask {
    id: NodeId,
    sender: mpsc::Sender<LeaderFollowerMessage>,
    receiver: mpsc::Receiver<LeaderFollowerMessage>,
    reader: Option<FollowerToLeaderMessageStreamReader>,
    writer: Option<LeaderToFollowerMessageStreamWriter>,
    writer_message_buffer: Vec<LeaderToFollowerMessage>,
    leader_sender: mpsc::Sender<LeaderMessage>,
}

impl LeaderFollowerTask {
    pub fn new(
        id: NodeId,
        sender: mpsc::Sender<LeaderFollowerMessage>,
        receiver: mpsc::Receiver<LeaderFollowerMessage>,
        reader: Option<FollowerToLeaderMessageStreamReader>,
        writer: Option<LeaderToFollowerMessageStreamWriter>,
        leader_sender: mpsc::Sender<LeaderMessage>,
    ) -> Self {
        Self {
            id,
            receiver,
            sender,
            reader,
            writer,
            writer_message_buffer: Vec::default(),
            leader_sender,
        }
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await
        });
    }

    async fn run(mut self) {
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
                .on_replicate(term, last_committed_log_entry_id, log_entry_data, sender)
                .await
                .expect(""),
            LeaderFollowerMessage::Handshake {
                reader,
                writer,
                result_sender,
            } => {
                self.on_handshake(reader, writer, result_sender).await;
            }
            LeaderFollowerMessage::Status {
                sender,
            } => {
                self.on_status(sender).await;
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
        result_sender: oneshot::Sender<Result<(), String>>,
    ) -> Result<(), Box<dyn Error>> {
        if self.reader.is_none() && self.writer.is_none() {
            self.reader = Some(reader);
            self.writer = Some(writer);

            if result_sender.send(Ok(())).is_err() {
                return Err("Cannot send result".into())
            }
        } else {
            if result_sender.send(Err("Follower is online".into())).is_err() {
                return Err("Cannot send result".into())
            }
        }

        Ok(())


    }

    async fn on_status(
        &mut self,
        sender: oneshot::Sender<LeaderFollowerStatus>,
    ) {
        sender.send(LeaderFollowerStatus::new(
            self.reader.is_some() && self.writer.is_some()
        ));
    }

    async fn on_replicate(
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
