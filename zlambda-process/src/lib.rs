/*use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use tokio::process::Command;
use zlambda_common::async_trait::async_trait;
use zlambda_common::module::{
    DispatchModuleEventError, DispatchModuleEventInput, DispatchModuleEventOutput,
    InitializeModuleEventError, InitializeModuleEventInput, InitializeModuleEventOutput,
    ModuleEventListener,
};
use zlambda_common::module_event_listener;

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
    async fn initialize(
        _input: InitializeModuleEventInput,
    ) -> Result<InitializeModuleEventOutput, InitializeModuleEventError> {
        Ok(InitializeModuleEventOutput::new(Box::new(Self {})))
    }

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

        Ok(DispatchModuleEventOutput::new(stdout.into()))
    }
}*/

use serde::{Deserialize, Serialize};
use tokio::process::Command;
use zlambda_core::common::async_trait;
use zlambda_core::common::module::{Module};
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessDispatcherNotificationHeader {
    program: String,
    arguments: Vec<String>,
}

impl From<ProcessDispatcherNotificationHeader> for (String, Vec<String>) {
    fn from(header: ProcessDispatcherNotificationHeader) -> Self {
        (header.program, header.arguments)
    }
}

impl ProcessDispatcherNotificationHeader {
    pub fn new(
        program: String,
        arguments: Vec<String>,
    ) -> Self {
        Self { program, arguments }
    }

    pub fn program(&self) -> &String {
        &self.program
    }

    pub fn arguments(&self) -> &Vec<String> {
        &self.arguments
    }
}


////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct ProcessDispatcher {}

#[async_trait]
impl Module for ProcessDispatcher {}

#[async_trait]
impl ServerModule for ProcessDispatcher {
    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (_server, _source, notification_body_item_queue_receiver) = input.into();

        let mut deserializer = notification_body_item_queue_receiver.deserializer();
        let header = deserializer
            .deserialize::<ProcessDispatcherNotificationHeader>()
            .await
            .unwrap();

        let stdout = Command::new(header.program)
            .args(header.arguments)
            .output()
            .await
            .unwrap()
            .stdout;

        println!("{:?}", std::str::from_utf8(&stdout));
    }
}
