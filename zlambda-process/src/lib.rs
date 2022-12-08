use serde::{Deserialize, Serialize};
use zlambda_common::async_trait::async_trait;
use zlambda_common::module::ModuleEventListener;
use zlambda_common::module::{
    DispatchModuleEventError, DispatchModuleEventInput, DispatchModuleEventOutput,
    ReadModuleEventError, ReadModuleEventInput, ReadModuleEventOutput, WriteModuleEventError,
    WriteModuleEventInput, WriteModuleEventOutput,
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
    async fn read(
        &self,
        event: ReadModuleEventInput,
    ) -> Result<ReadModuleEventOutput, ReadModuleEventError> {
        let (arguments,) = event.into();

        todo!()
    }

    async fn write(
        &self,
        event: WriteModuleEventInput,
    ) -> Result<WriteModuleEventOutput, WriteModuleEventError> {
        todo!()
    }

    async fn dispatch(
        &self,
        event: DispatchModuleEventInput,
    ) -> Result<DispatchModuleEventOutput, DispatchModuleEventError> {
        todo!()
    }
}

#[no_mangle]
pub extern "C" fn module_event_listener() -> Box<dyn ModuleEventListener> {
    Box::new(EventListener {})
}
