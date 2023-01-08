use crate::dispatch::DispatchId;
use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::module::ModuleId;
use crate::Bytes;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeToClientInitializeResponseMessage {
    module_id: ModuleId,
}

impl From<NodeToClientInitializeResponseMessage> for (ModuleId,) {
    fn from(message: NodeToClientInitializeResponseMessage) -> Self {
        (message.module_id,)
    }
}

impl NodeToClientInitializeResponseMessage {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeToClientLoadResponseMessage {
    module_id: ModuleId,
    result: Result<(), String>,
}

impl From<NodeToClientLoadResponseMessage> for (ModuleId, Result<(), String>) {
    fn from(message: NodeToClientLoadResponseMessage) -> Self {
        (message.module_id, message.result)
    }
}

impl NodeToClientLoadResponseMessage {
    pub fn new(module_id: ModuleId, result: Result<(), String>) -> Self {
        Self { module_id, result }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn result(&self) -> &Result<(), String> {
        &self.result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeToClientApplyResponseMessage {
    module_id: ModuleId,
    result: Result<(), String>,
}

impl From<NodeToClientApplyResponseMessage> for (ModuleId, Result<(), String>) {
    fn from(message: NodeToClientApplyResponseMessage) -> Self {
        (message.module_id, message.result)
    }
}

impl NodeToClientApplyResponseMessage {
    pub fn new(module_id: ModuleId, result: Result<(), String>) -> Self {
        Self { module_id, result }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }

    pub fn result(&self) -> &Result<(), String> {
        &self.result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeToClientDispatchResponseMessage {
    dispatch_id: DispatchId,
    result: Result<Bytes, String>,
}

impl From<NodeToClientDispatchResponseMessage> for (DispatchId, Result<Bytes, String>) {
    fn from(message: NodeToClientDispatchResponseMessage) -> Self {
        (message.dispatch_id, message.result)
    }
}

impl NodeToClientDispatchResponseMessage {
    pub fn new(dispatch_id: DispatchId, result: Result<Bytes, String>) -> Self {
        Self {
            dispatch_id,
            result,
        }
    }

    pub fn dispatch_id(&self) -> DispatchId {
        self.dispatch_id
    }

    pub fn result(&self) -> &Result<Bytes, String> {
        &self.result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeToClientMessage {
    InitializeResponse(NodeToClientInitializeResponseMessage),
    LoadResponse(NodeToClientLoadResponseMessage),
    ApplyResponse(NodeToClientApplyResponseMessage),
    DispatchResponse {
        dispatch_id: DispatchId,
        result: Result<Bytes, String>,
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
