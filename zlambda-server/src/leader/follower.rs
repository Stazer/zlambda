use crate::leader::LeaderHandle;
use std::error::Error;
use std::mem::take;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::channel::{DoReceive, DoSend};
use zlambda_common::log::{LogEntryData, LogEntryId};
use zlambda_common::message::{
    FollowerToLeaderMessage, FollowerToLeaderMessageStreamReader, GuestToLeaderMessageStreamReader,
    LeaderToFollowerMessage, LeaderToFollowerMessageStreamWriter, LeaderToGuestMessage,
    LeaderToGuestMessageStreamWriter, MessageError,
};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum LeaderFollowerResult {
    Continue,
    Stop,
    InternalError(Box<dyn Error>),
    ClientError(Box<dyn Error>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderFollowerReplicateMessage {
    term: Term,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_entry_data: Vec<LogEntryData>,
    sender: oneshot::Sender<()>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderFollowerStatusMessage {
    sender: oneshot::Sender<LeaderFollowerStatus>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderFollowerHandshakeMessage {
    reader: GuestToLeaderMessageStreamReader,
    writer: LeaderToGuestMessageStreamWriter,
    sender: oneshot::Sender<Result<(), String>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum LeaderFollowerMessage {
    Replicate(LeaderFollowerReplicateMessage),
    Status(LeaderFollowerStatusMessage),
    Handshake(LeaderFollowerHandshakeMessage),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct LeaderFollowerHandle {
    sender: mpsc::Sender<LeaderFollowerMessage>,
}

impl LeaderFollowerHandle {
    fn new(sender: mpsc::Sender<LeaderFollowerMessage>) -> Self {
        Self { sender }
    }

    pub async fn status(&self) -> LeaderFollowerStatus {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderFollowerMessage::Status(LeaderFollowerStatusMessage {
                sender,
            }))
            .await;

        receiver.do_receive().await
    }

    pub async fn replicate(
        &self,
        term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
    ) {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderFollowerMessage::Replicate(
                LeaderFollowerReplicateMessage {
                    term,
                    last_committed_log_entry_id,
                    log_entry_data,
                    sender,
                },
            ))
            .await;

        receiver.do_receive().await;
    }

    pub async fn handshake(
        &self,
        reader: GuestToLeaderMessageStreamReader,
        writer: LeaderToGuestMessageStreamWriter,
    ) -> Result<(), Box<dyn Error>> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderFollowerMessage::Handshake(
                LeaderFollowerHandshakeMessage {
                    reader: reader.into(),
                    writer: writer.into(),
                    sender,
                },
            ))
            .await;

        receiver.do_receive().await.expect("");

        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderFollowerBuilder {
    sender: mpsc::Sender<LeaderFollowerMessage>,
    receiver: mpsc::Receiver<LeaderFollowerMessage>,
}

impl LeaderFollowerBuilder {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(16);

        Self { sender, receiver }
    }

    pub fn handle(&self) -> LeaderFollowerHandle {
        LeaderFollowerHandle::new(self.sender.clone())
    }

    pub fn build(
        self,
        id: NodeId,
        reader: Option<FollowerToLeaderMessageStreamReader>,
        writer: Option<LeaderToFollowerMessageStreamWriter>,
        leader_handle: LeaderHandle,
    ) -> LeaderFollowerTask {
        LeaderFollowerTask::new(id, self.receiver, reader, writer, leader_handle)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollowerStatus {
    available: bool,
}

impl LeaderFollowerStatus {
    pub fn new(available: bool) -> Self {
        Self { available }
    }

    pub fn available(&self) -> bool {
        self.available
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollowerTask {
    id: NodeId,
    receiver: mpsc::Receiver<LeaderFollowerMessage>,
    reader: Option<FollowerToLeaderMessageStreamReader>,
    writer: Option<LeaderToFollowerMessageStreamWriter>,
    writer_message_buffer: Vec<LeaderToFollowerMessage>,
    leader_handle: LeaderHandle,
}

impl LeaderFollowerTask {
    fn new(
        id: NodeId,
        receiver: mpsc::Receiver<LeaderFollowerMessage>,
        reader: Option<FollowerToLeaderMessageStreamReader>,
        writer: Option<LeaderToFollowerMessageStreamWriter>,
        leader_handle: LeaderHandle,
    ) -> Self {
        Self {
            id,
            receiver,
            reader,
            writer,
            writer_message_buffer: Vec::default(),
            leader_handle,
        }
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    async fn run(mut self) {
        loop {
            self.select().await;
        }
    }

    async fn select(&mut self) {
        match self.reader {
            Some(ref mut reader) => {
                select!(
                    read_result = reader.read() => {
                        self.on_read_result(read_result).await;
                    }
                    receive_result = self.receiver.recv() => {
                        self.on_receive_result(receive_result).await;
                    }
                );
            }
            None => {
                select!(
                    receive_result = self.receiver.recv() => {
                        self.on_receive_result(receive_result).await;
                    }
                );
            }
        };
    }

    async fn on_read_result(
        &mut self,
        result: Result<Option<FollowerToLeaderMessage>, MessageError>,
    ) {
        match result {
            Ok(None) => {
                self.reader = None;
                self.writer = None;
            }
            Ok(Some(message)) => {
                self.on_registered_follower_to_leader_message(message).await;
            }
            Err(error) => {
                error!("{}", error);
            }
        };
    }

    async fn on_receive_result(&mut self, result: Option<LeaderFollowerMessage>) {
        if let Some(message) = result {
            self.on_message(message).await;
        }
    }

    async fn on_message(&mut self, message: LeaderFollowerMessage) {
        match message {
            LeaderFollowerMessage::Replicate(message) => {
                self.on_replicate(message).await.expect("")
            }
            LeaderFollowerMessage::Handshake(message) => {
                self.on_handshake(message).await.expect("")
            }
            LeaderFollowerMessage::Status(message) => self.on_status(message).await,
        }
    }

    async fn on_registered_follower_to_leader_message(&mut self, message: FollowerToLeaderMessage) {
        match message {
            FollowerToLeaderMessage::AppendEntriesResponse {
                appended_log_entry_ids,
                missing_log_entry_ids,
            } => {
                self.leader_handle
                    .acknowledge(appended_log_entry_ids, self.id)
                    .await;
            }
        }
    }

    async fn on_handshake(
        &mut self,
        mut message: LeaderFollowerHandshakeMessage,
    ) -> Result<(), Box<dyn Error>> {
        if self.reader.is_none() && self.writer.is_none() {
            message
                .writer
                .write(LeaderToGuestMessage::HandshakeOkResponse { leader_id: 0 })
                .await
                .expect("");

            let mut writer = message.writer.into();

            let mut iterator = take(&mut self.writer_message_buffer).into_iter();

            while let Some(message) = iterator.next() {
                if let Err(error) = writer.write(message).await {
                    error!("{}", error);
                    self.writer_message_buffer = iterator.collect();
                    break;
                }
            }

            self.reader = Some(message.reader.into());
            self.writer = Some(writer);

            message.sender.do_send(Ok(())).await;
        } else {
            message
                .sender
                .do_send(Err("Follower is online".into()))
                .await;
        }

        Ok(())
    }

    async fn on_status(&mut self, message: LeaderFollowerStatusMessage) {
        message
            .sender
            .do_send(LeaderFollowerStatus::new(
                self.reader.is_some() && self.writer.is_some(),
            ))
            .await
    }

    async fn on_replicate(
        &mut self,
        message: LeaderFollowerReplicateMessage,
    ) -> Result<(), Box<dyn Error>> {
        let request = LeaderToFollowerMessage::AppendEntriesRequest {
            term: message.term,
            last_committed_log_entry_id: message.last_committed_log_entry_id,
            log_entry_data: message.log_entry_data,
        };

        if let Some(ref mut writer) = &mut self.writer {
            writer.write(request).await?;
        } else {
            self.writer_message_buffer.push(request);
        }

        message.sender.do_send(()).await;

        Ok(())
    }
}
