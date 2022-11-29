use crate::async_trait::async_trait;
use crate::message::ClientMessageDispatchPayload;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CreateDispatchPayloadEvent {
    arguments: Vec<String>,
}

impl From<CreateDispatchPayloadEvent> for (Vec<String>,) {
    fn from(event: CreateDispatchPayloadEvent) -> Self {
        (event.arguments,)
    }
}

impl CreateDispatchPayloadEvent {
    pub fn new(arguments: Vec<String>) -> Self {
        Self { arguments }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CreateDispatchPayloadEventResult {
    payload: ClientMessageDispatchPayload,
}

impl From<CreateDispatchPayloadEventResult> for (ClientMessageDispatchPayload,) {
    fn from(result: CreateDispatchPayloadEventResult) -> Self {
        (result.payload,)
    }
}

impl CreateDispatchPayloadEventResult {
    pub fn new(payload: ClientMessageDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DispatchEvent {
    payload: ClientMessageDispatchPayload,
}

impl From<DispatchEvent> for (ClientMessageDispatchPayload,) {
    fn from(event: DispatchEvent) -> Self {
        (event.payload,)
    }
}

impl DispatchEvent {
    pub fn new(payload: ClientMessageDispatchPayload) -> Self {
        Self { payload }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DispatchEventResult {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait ModuleEventListener: Send + Sync {
    async fn on_create_dispatch_payload(
        &self,
        _event: CreateDispatchPayloadEvent,
    ) -> CreateDispatchPayloadEventResult;
    async fn on_dispatch(&self, _event: DispatchEvent) -> DispatchEventResult;
}
