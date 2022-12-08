use serde::{Deserialize, Serialize};
use zlambda_common::async_trait::async_trait;
use zlambda_common::module::{
    DispatchModuleEventError, DispatchModuleEventInput, DispatchModuleEventOutput,
    ModuleEventDispatchPayload, ModuleEventListener, ReadModuleEventError, ReadModuleEventInput,
    ReadModuleEventOutput, WriteModuleEventError, WriteModuleEventInput, WriteModuleEventOutput,
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

        let mut iterator = arguments.into_iter();

        let program = match iterator.next() {
            Some(program) => program,
            None => return Err("Program should be given".into()),
        };

        let arguments = iterator.collect();

        Ok(ReadModuleEventOutput::new(ModuleEventDispatchPayload::new(
            &DispatchPayload {
                program,
                arguments,
            },
        )?))
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
        let (payload,) = event.into();
        let dispatch_payload = payload.into_inner::<DispatchPayload>();

        println!("{:?}", dispatch_payload);

        Ok(DispatchModuleEventOutput::new(
            ModuleEventDispatchPayload::default(),
        ))
    }
}

#[no_mangle]
pub extern "C" fn module_event_listener() -> Box<dyn ModuleEventListener> {
    Box::new(EventListener {})
}
