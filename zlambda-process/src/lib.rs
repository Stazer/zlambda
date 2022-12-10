use serde::{Deserialize, Serialize};
use zlambda_common::async_trait::async_trait;
use zlambda_common::module::{
    ModuleEventListener,
    DispatchModuleEventError, DispatchModuleEventInput, DispatchModuleEventOutput,
};
use serde_json::from_slice;

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
    async fn dispatch(
        &self,
        event: DispatchModuleEventInput,
    ) -> Result<DispatchModuleEventOutput, DispatchModuleEventError> {
        let (payload,) = event.into();
        let payload = from_slice::<DispatchPayload>(&payload)
            .map_err(|e| DispatchModuleEventError::from(Box::from(e)))?;

        Ok(DispatchModuleEventOutput::new(Vec::new()))
    }
}

#[no_mangle]
pub extern "C" fn module_event_listener() -> Box<dyn ModuleEventListener> {
    Box::new(EventListener {})
}
