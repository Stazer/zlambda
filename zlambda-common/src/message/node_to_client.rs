use crate::dispatch::DispatchId;
use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::module::ModuleId;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeToClientMessage {
    InitializeResponse {
        module_id: ModuleId,
    },
    LoadResponse {
        module_id: ModuleId,
        result: Result<(), String>,
    },
    ApplyResponse {
        module_id: ModuleId,
        result: Result<(), String>,
    },
    DispatchResponse {
        dispatch_id: DispatchId,
        result: Result<Vec<u8>, String>,
    },
}

impl From<Message> for Result<NodeToClientMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::NodeToClient(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NodeToClientMessageStreamReader = BasicMessageStreamReader<NodeToClientMessage>;
pub type NodeToClientMessageStreamWriter = BasicMessageStreamWriter<NodeToClientMessage>;
