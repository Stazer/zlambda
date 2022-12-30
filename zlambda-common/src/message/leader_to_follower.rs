use crate::message::{Message, MessageError, BasicMessageStreamReader, BasicMessageStreamWriter};
use serde::{Deserialize, Serialize};
use crate::log::{LogEntryId, LogEntryData};
use crate::term::{Term};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LeaderToFollowerMessage {
    AppendEntriesRequest {
        term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        //last_appended_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
    },
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
