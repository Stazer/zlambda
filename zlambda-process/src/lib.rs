use serde::{Deserialize, Serialize};
use zlambda_common::async_trait::async_trait;
use zlambda_common::module::ModuleEventListener;
use zlambda_common::module::{
    ExecuteModuleEventInput, ExecuteModuleEventOutput, RunModuleEventInput, RunModuleEventOutput,
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
    async fn run(&self, event: RunModuleEventInput) -> RunModuleEventOutput {
        todo!()
    }

    async fn execute(&self, event: ExecuteModuleEventInput) -> ExecuteModuleEventOutput {
        todo!()
    }
}

#[no_mangle]
pub extern "C" fn module_event_listener() -> Box<dyn ModuleEventListener> {
    Box::new(EventListener {})
}
