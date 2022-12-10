use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use tokio::process::Command;
use tokio::spawn;
use zlambda_common::async_trait::async_trait;
use zlambda_common::module::{
    DispatchModuleEventError, DispatchModuleEventInput, DispatchModuleEventOutput,
    ModuleEventListener,
    ModuleEventHandler,
    SimpleModuleEventHandler,
    async_ffi,
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

struct EventHandler(EventListener);

use async_ffi::FutureExt;

impl ModuleEventHandler for EventHandler {
    fn dispatch(
        &self,
        handle: tokio::runtime::Handle,
        input: DispatchModuleEventInput,
    ) -> async_ffi::BorrowingFfiFuture<Result<DispatchModuleEventOutput, DispatchModuleEventError>> {
        let future = self.0.dispatch(input);

        async move {
            let _enter = handle.enter();
            future.await
        }.into_ffi()
    }
}

#[no_mangle]
pub extern "C" fn module_event_listener() -> Box<dyn ModuleEventHandler> {
    Box::new(SimpleModuleEventHandler::new(Box::new(EventListener{})))
}
