use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use tokio::process::Command;
use zlambda_common::async_trait::async_trait;
use zlambda_common::module::{
    DefaultModuleEventHandler, DispatchModuleEventError, DispatchModuleEventInput,
    DispatchModuleEventOutput, ModuleEventHandler, ModuleEventListener,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
struct DispatchPayload {
    program: String,
    arguments: Vec<String>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct EventListener {}

#[async_trait]
impl ModuleEventListener for EventListener {
    async fn dispatch(
        &self,
        event: DispatchModuleEventInput,
    ) -> Result<DispatchModuleEventOutput, DispatchModuleEventError> {
        let (payload,) = event.into();

        let payload = from_slice::<DispatchPayload>(&payload)
            .map_err(|e| DispatchModuleEventError::from(Box::from(e)))
            .unwrap();

        let stdout = Command::new(payload.program)
            .args(payload.arguments)
            .output()
            .await
            .map_err(|e| DispatchModuleEventError::from(Box::from(e)))
            .unwrap()
            .stdout;

        Ok(DispatchModuleEventOutput::new(stdout))
    }
}

#[no_mangle]
pub extern "C" fn module_event_handler() -> Box<dyn ModuleEventHandler> {
    Box::new(DefaultModuleEventHandler::new(Box::new(EventListener {})))
}
