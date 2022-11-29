use serde::{Deserialize, Serialize};
use zlambda_common::async_trait::async_trait;
use zlambda_common::module::ModuleEventListener;
use zlambda_common::module::{
    CreateDispatchPayloadEvent, CreateDispatchPayloadEventResult, DispatchEvent,
    DispatchEventResult,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
struct DispatchPayload {
    program: String,
    arguments: Vec<String>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct EventListener {}

#[async_trait]
impl ModuleEventListener for EventListener {
    async fn on_create_dispatch_payload(
        &self,
        _event: CreateDispatchPayloadEvent,
    ) -> CreateDispatchPayloadEventResult {
        todo!()
    }

    async fn on_dispatch(&self, event: DispatchEvent) -> DispatchEventResult {
        //let (payload,) = event.into();
        println!("{:?}", event);

        todo!()
    }
}

#[no_mangle]
pub extern "C" fn module_event_listener() -> Box<dyn ModuleEventListener> {
    Box::new(EventListener {})
}
