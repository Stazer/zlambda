use serde::{Deserialize, Serialize};
use zlambda_common::async_trait::async_trait;
use zlambda_common::module::{
    ModuleEventListener,
    DispatchModuleEventError, DispatchModuleEventInput, DispatchModuleEventOutput,
};
use serde_json::from_slice;
use tokio::process::Command;
use tokio::spawn;

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

        spawn(async move {
            let stdout = Command::new(payload.program).args(payload.arguments).output().await
            .map_err(|e| DispatchModuleEventError::from(Box::from(e))).unwrap()
            .stdout;

            println!("{:?}", stdout);
        });

        Ok(DispatchModuleEventOutput::new(Vec::new()))
    }
}

#[no_mangle]
pub extern "C" fn module_event_listener() -> Box<dyn ModuleEventListener> {
    Box::new(EventListener {})
}
