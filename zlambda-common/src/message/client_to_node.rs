use crate::dispatch::DispatchId;
use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::module::ModuleId;
use crate::node::NodeId;
use bytes::Bytes;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ClientToNodeInitializeRequestMessage {}

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

impl ClientToNodeAppendMessage {
    pub fn new(
        module_id: ModuleId,
        bytes: Bytes,
    ) -> Self {
        Self {
            module_id,
            bytes,
        }
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

impl ClientToNodeLoadRequestMessage {
    pub fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
        }
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
    payload: Vec<u8>,
    node_id: Option<NodeId>,
}

impl ClientToNodeDispatchRequestMessage {
    pub fn new(
        dispatch_id: DispatchId,
        module_id: ModuleId,
        payload: Vec<u8>,
        node_id: Option<NodeId>,
    ) -> Self {
        Self {
            dispatch_id,
            module_id,
            payload,
            node_id,
        }
    }

    pub fn dispatch_id(&self) -> DispatchId  {
        self.dispatch_id
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn payload(&self) -> &Vec<u8> {
        &self.payload
    }

    pub fn node_id(&self) -> Option<NodeId> {
        self.node_id
    }
}

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
