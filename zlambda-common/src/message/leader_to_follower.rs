use crate::dispatch::DispatchId;
use crate::log::{LogEntryData, LogEntryId};
use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::term::Term;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaderToFollowerAppendEntriesRequestMessage {
    term: Term,
    last_committed_log_entry_id: Option<LogEntryId>,
    log_entry_data: Vec<LogEntryData>,
}

impl From<LeaderToFollowerAppendEntriesRequestMessage>
    for (Term, Option<LogEntryId>, Vec<LogEntryData>)
{
    fn from(message: LeaderToFollowerAppendEntriesRequestMessage) -> Self {
        (
            message.term,
            message.last_committed_log_entry_id,
            message.log_entry_data,
        )
    }
}

impl LeaderToFollowerAppendEntriesRequestMessage {
    pub fn new(
        term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
    ) -> Self {
        Self {
            term,
            last_committed_log_entry_id,
            log_entry_data,
        }
    }

    pub fn term(&self) -> Term {
        self.term
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }

    pub fn log_entry_data(&self) -> &Vec<LogEntryData> {
        &self.log_entry_data
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaderToFollowerDispatchResponseMessage {
    dispatch_id: DispatchId,
    result: Result<Vec<u8>, String>,
}

impl From<LeaderToFollowerDispatchResponseMessage> for (DispatchId, Result<Vec<u8>, String>) {
    fn from(message: LeaderToFollowerDispatchResponseMessage) -> Self {
        (message.dispatch_id, message.result)
    }
}

impl LeaderToFollowerDispatchResponseMessage {
    pub fn new(dispatch_id: DispatchId, result: Result<Vec<u8>, String>) -> Self {
        Self {
            dispatch_id,
            result,
        }
    }

    pub fn dispatch_id(&self) -> DispatchId {
        self.dispatch_id
    }

    pub fn result(&self) -> &Result<Vec<u8>, String> {
        &self.result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LeaderToFollowerMessage {
    AppendEntriesRequest(LeaderToFollowerAppendEntriesRequestMessage),
    DispatchResponse(LeaderToFollowerDispatchResponseMessage),
}

impl From<Message> for Result<LeaderToFollowerMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::LeaderToFollower(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LeaderToFollowerMessageStreamReader = BasicMessageStreamReader<LeaderToFollowerMessage>;
pub type LeaderToFollowerMessageStreamWriter = BasicMessageStreamWriter<LeaderToFollowerMessage>;
