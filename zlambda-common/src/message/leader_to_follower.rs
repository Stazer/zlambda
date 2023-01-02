use crate::log::{LogEntryData, LogEntryId};
use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::term::Term;
use serde::{Deserialize, Serialize};
use crate::dispatch::DispatchId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LeaderToFollowerMessage {
    AppendEntriesRequest {
        term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        //last_appended_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
    },
    DispatchResponse {
        id: DispatchId,
        payload: Vec<u8>,
    }
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
