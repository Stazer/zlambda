use crate::dispatch::DispatchId;
use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::module::ModuleId;
use crate::node::NodeId;
use crate::Bytes;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToNodeInitializeRequestMessage {}

impl From<ClientToNodeInitializeRequestMessage> for () {
    fn from(message: ClientToNodeInitializeRequestMessage) -> Self {
        ()
    }
}

impl ClientToNodeInitializeRequestMessage {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToNodeAppendMessage {
    module_id: ModuleId,
    bytes: Bytes,
}

impl From<ClientToNodeAppendMessage> for (ModuleId, Bytes) {
    fn from(message: ClientToNodeAppendMessage) -> Self {
        (message.module_id, message.bytes)
    }
}

impl ClientToNodeAppendMessage {
    pub fn new(module_id: ModuleId, bytes: Bytes) -> Self {
        Self { module_id, bytes }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn bytes(&self) -> &Bytes {
        &self.bytes
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToNodeLoadRequestMessage {
    module_id: ModuleId,
}

impl From<ClientToNodeLoadRequestMessage> for (ModuleId,) {
    fn from(message: ClientToNodeLoadRequestMessage) -> Self {
        (message.module_id,)
    }
}

impl ClientToNodeLoadRequestMessage {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToNodeDispatchRequestMessage {
    dispatch_id: DispatchId,
    module_id: ModuleId,
    payload: Bytes,
    target_node_id: Option<NodeId>,
}

impl From<ClientToNodeDispatchRequestMessage> for (DispatchId, ModuleId, Bytes, Option<NodeId>) {
    fn from(message: ClientToNodeDispatchRequestMessage) -> Self {
        (
            message.dispatch_id,
            message.module_id,
            message.payload,
            message.target_node_id,
        )
    }
}

impl ClientToNodeDispatchRequestMessage {
    pub fn new(
        dispatch_id: DispatchId,
        module_id: ModuleId,
        payload: Bytes,
        target_node_id: Option<NodeId>,
    ) -> Self {
        Self {
            dispatch_id,
            module_id,
            payload,
            target_node_id,
        }
    }

    pub fn dispatch_id(&self) -> DispatchId {
        self.dispatch_id
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn payload(&self) -> &Bytes {
        &self.payload
    }

    pub fn target_node_id(&self) -> Option<NodeId> {
        self.target_node_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ClientToNodeMessage {
    InitializeRequest(ClientToNodeInitializeRequestMessage),
    Append(ClientToNodeAppendMessage),
    LoadRequest(ClientToNodeLoadRequestMessage),
    DispatchRequest(ClientToNodeDispatchRequestMessage),
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
