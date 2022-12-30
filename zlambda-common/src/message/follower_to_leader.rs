use crate::message::{Message, MessageError, BasicMessageStreamReader, BasicMessageStreamWriter};
use serde::{Deserialize, Serialize};
use crate::log::LogEntryId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FollowerToLeaderMessage {
    AppendEntriesResponse {
        appended_log_entry_ids: Vec<LogEntryId>,
        //acknowledged_log_entry_ids: Vec<LogEntryId>,
        missing_log_entry_ids: Vec<LogEntryId>,
    },
}

impl From<Message> for Result<FollowerToLeaderMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::FollowerToLeader(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type FollowerToLeaderMessageStreamReader = BasicMessageStreamReader<FollowerToLeaderMessage>;
pub type FollowerToLeaderMessageStreamWriter = BasicMessageStreamWriter<FollowerToLeaderMessage>;
