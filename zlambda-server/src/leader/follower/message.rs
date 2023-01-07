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
    term: Term,
    acknowledging_log_entry_data: Vec<LogEntryData>,
    last_committed_log_entry_id: Option<LogEntryId>,
    sender: oneshot::Sender<Result<(), String>>,
}

impl From<LeaderFollowerHandshakeMessage>
    for (
        GuestToLeaderMessageStreamReader,
        LeaderToGuestMessageStreamWriter,
        Term,
        Vec<LogEntryData>,
        Option<LogEntryId>,
        oneshot::Sender<Result<(), String>>,
    )
{
    fn from(message: LeaderFollowerHandshakeMessage) -> Self {
        (
            message.reader,
            message.writer,
            message.term,
            message.acknowledging_log_entry_data,
            message.last_committed_log_entry_id,
            message.sender,
        )
    }
}

impl LeaderFollowerHandshakeMessage {
    pub fn new(
        reader: GuestToLeaderMessageStreamReader,
        writer: LeaderToGuestMessageStreamWriter,
        term: Term,
        acknowledging_log_entry_data: Vec<LogEntryData>,
        last_committed_log_entry_id: Option<LogEntryId>,
        sender: oneshot::Sender<Result<(), String>>,
    ) -> Self {
        Self {
            reader,
            writer,
            term,
            acknowledging_log_entry_data,
            last_committed_log_entry_id,
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
