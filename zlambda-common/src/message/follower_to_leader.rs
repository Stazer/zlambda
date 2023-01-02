use crate::log::LogEntryId;
use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use serde::{Deserialize, Serialize};
use crate::module::ModuleId;
use crate::node::NodeId;
use crate::dispatch::DispatchId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FollowerToLeaderMessage {
    AppendEntriesResponse {
        appended_log_entry_ids: Vec<LogEntryId>,
        //acknowledged_log_entry_ids: Vec<LogEntryId>,
        missing_log_entry_ids: Vec<LogEntryId>,
    },
    DispatchRequest {
        id: DispatchId,
        module_id: ModuleId,
        payload: Vec<u8>,
        node_id: Option<NodeId>,
    }
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
