use crate::async_trait::async_trait;
use crate::message::ClientMessageDispatchPayload;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CreateDispatchPayloadModuleEvent {
    arguments: Vec<String>,
}

impl From<CreateDispatchPayloadModuleEvent> for (Vec<String>,) {
    fn from(event: CreateDispatchPayloadModuleEvent) -> Self {
        (event.arguments,)
    }
}

impl CreateDispatchPayloadModuleEvent {
    pub fn new(arguments: Vec<String>) -> Self {
        Self { arguments }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CreateDispatchPayloadModuleEventResult {
    payload: ClientMessageDispatchPayload,
}

impl From<CreateDispatchPayloadModuleEventResult> for (ClientMessageDispatchPayload,) {
    fn from(result: CreateDispatchPayloadModuleEventResult) -> Self {
        (result.payload,)
    }
}

impl CreateDispatchPayloadModuleEventResult {
    pub fn new(payload: ClientMessageDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DispatchModuleEvent {
    payload: ClientMessageDispatchPayload,
}

impl From<DispatchModuleEvent> for (ClientMessageDispatchPayload,) {
    fn from(event: DispatchModuleEvent) -> Self {
        (event.payload,)
    }
}

impl DispatchModuleEvent {
    pub fn new(payload: ClientMessageDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DispatchModuleEventResult {}

impl DispatchModuleEventResult {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait ModuleEventListener: Send + Sync {
    async fn on_create_dispatch_payload(
        &self,
        _event: CreateDispatchPayloadModuleEvent,
    ) -> CreateDispatchPayloadModuleEventResult;
    async fn on_dispatch(&self, _event: DispatchModuleEvent) -> DispatchModuleEventResult;
}
