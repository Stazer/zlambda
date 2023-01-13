use crate::dispatch::DispatchId;
use crate::log::LogEntryId;
use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::module::ModuleId;
use crate::node::NodeId;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FollowerToLeaderAppendEntriesResponseMessage {
    appended_log_entry_ids: Vec<LogEntryId>,
    missing_log_entry_ids: Vec<LogEntryId>,
}

impl FollowerToLeaderAppendEntriesResponseMessage {
    pub fn new(
        appended_log_entry_ids: Vec<LogEntryId>,
        missing_log_entry_ids: Vec<LogEntryId>,
    ) -> Self {
        Self {
            appended_log_entry_ids,
            missing_log_entry_ids,
        }
    }
}

impl From<FollowerToLeaderAppendEntriesResponseMessage> for (Vec<LogEntryId>, Vec<LogEntryId>) {
    fn from(message: FollowerToLeaderAppendEntriesResponseMessage) -> Self {
        (
            message.appended_log_entry_ids,
            message.missing_log_entry_ids,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FollowerToLeaderDispatchRequestMessage {
    id: DispatchId,
    module_id: ModuleId,
    payload: Vec<u8>,
    node_id: Option<NodeId>,
}

impl FollowerToLeaderDispatchRequestMessage {
    pub fn new(
        id: DispatchId,
        module_id: ModuleId,
        payload: Vec<u8>,
        node_id: Option<NodeId>,
    ) -> Self {
        Self {
            id,
            module_id,
            payload,
            node_id,
        }
    }
}

impl From<FollowerToLeaderDispatchRequestMessage>
    for (DispatchId, ModuleId, Vec<u8>, Option<NodeId>)
{
    fn from(message: FollowerToLeaderDispatchRequestMessage) -> Self {
        (
            message.id,
            message.module_id,
            message.payload,
            message.node_id,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum FollowerToLeaderMessage {
    AppendEntriesResponse(FollowerToLeaderAppendEntriesResponseMessage),
    DispatchRequest(FollowerToLeaderDispatchRequestMessage),
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
