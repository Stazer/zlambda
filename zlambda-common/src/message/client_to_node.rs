use crate::message::{Message, MessageError, BasicMessageStreamReader, BasicMessageStreamWriter};
use serde::{Deserialize, Serialize};
use crate::dispatch::DispatchId;
use crate::module::ModuleId;
use bytes::Bytes;

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
        //node_id: Option<NodeId>,
        module_id: ModuleId,
        dispatch_id: DispatchId,
        payload: Vec<u8>,
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
