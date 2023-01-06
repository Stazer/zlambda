use tokio::sync::oneshot;
use zlambda_common::log::{LogEntryData, LogEntryId};
use zlambda_common::message::{GuestToLeaderMessageStreamReader, LeaderToGuestMessageStreamWriter};
use zlambda_common::term::Term;

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
pub struct LeaderFollowerReplicateMessage {
    term: Term,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_entry_data: Vec<LogEntryData>,
    sender: oneshot::Sender<()>,
}

impl From<LeaderFollowerReplicateMessage>
    for (
        Term,
        Option<LogEntryId>,
        Vec<LogEntryData>,
        oneshot::Sender<()>,
    )
{
    fn from(message: LeaderFollowerReplicateMessage) -> Self {
        (
            message.term,
            message.last_committed_log_entry_id,
            message.log_entry_data,
            message.sender,
        )
    }
}

impl LeaderFollowerReplicateMessage {
    pub fn new(
        term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
        sender: oneshot::Sender<()>,
    ) -> Self {
        Self {
            term,
            last_committed_log_entry_id,
            log_entry_data,
            sender,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollowerStatusMessage {
    sender: oneshot::Sender<LeaderFollowerStatus>,
}

impl From<LeaderFollowerStatusMessage> for (oneshot::Sender<LeaderFollowerStatus>,) {
    fn from(message: LeaderFollowerStatusMessage) -> Self {
        (message.sender,)
    }
}

impl LeaderFollowerStatusMessage {
    pub fn new(sender: oneshot::Sender<LeaderFollowerStatus>) -> Self {
        Self { sender }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollowerHandshakeMessage {
    reader: GuestToLeaderMessageStreamReader,
    writer: LeaderToGuestMessageStreamWriter,
    sender: oneshot::Sender<Result<(), String>>,
}

impl From<LeaderFollowerHandshakeMessage>
    for (
        GuestToLeaderMessageStreamReader,
        LeaderToGuestMessageStreamWriter,
        oneshot::Sender<Result<(), String>>,
    )
{
    fn from(message: LeaderFollowerHandshakeMessage) -> Self {
        (
            message.reader,
            message.writer,
            message.sender,
        )
    }
}

impl LeaderFollowerHandshakeMessage {
    pub fn new(
        reader: GuestToLeaderMessageStreamReader,
        writer: LeaderToGuestMessageStreamWriter,
        sender: oneshot::Sender<Result<(), String>>,
    ) -> Self {
        Self {
            reader,
            writer,
            sender,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LeaderFollowerMessage {
    Replicate(LeaderFollowerReplicateMessage),
    Status(LeaderFollowerStatusMessage),
    Handshake(LeaderFollowerHandshakeMessage),
}
