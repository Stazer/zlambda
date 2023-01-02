use crate::dispatch::DispatchId;
use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::module::ModuleId;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use crate::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClientToNodeMessage {
    InitializeRequest,
    Append {
        module_id: ModuleId,
        bytes: Bytes,
    },
    LoadRequest {
        module_id: ModuleId,
    },
    /*ApplyRequest {
        module_id: ModuleId,
    },*/
    DispatchRequest {
        dispatch_id: DispatchId,
        module_id: ModuleId,
        payload: Vec<u8>,
        node_id: Option<NodeId>,
    },
}

impl From<Message> for Result<ClientToNodeMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::ClientToNode(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientToNodeMessageStreamReader = BasicMessageStreamReader<ClientToNodeMessage>;
pub type ClientToNodeMessageStreamWriter = BasicMessageStreamWriter<ClientToNodeMessage>;
