use serde::{Deserialize, Serialize};
use tokio::process::Command;
use zlambda_core::common::async_trait;
use zlambda_core::common::module::{ModuleId, Module};
use zlambda_core::common::notification::{
    NotificationBodyItemStreamExt,
    notification_body_item_queue,
};
use zlambda_core::common::future::stream::StreamExt;
use zlambda_core::common::runtime::spawn;
use zlambda_core::common::bytes::Bytes;
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};
use std::process::Stdio;
use zlambda_core::common::io::{AsyncReadExt, AsyncWriteExt};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessDispatcherNotificationHeader {
    program: String,
    arguments: Vec<String>,
    module_id: ModuleId,
}

impl From<ProcessDispatcherNotificationHeader> for (String, Vec<String>, ModuleId) {
    fn from(header: ProcessDispatcherNotificationHeader) -> Self {
        (header.program, header.arguments, header.module_id)
    }
}

impl ProcessDispatcherNotificationHeader {
    pub fn new(program: String, arguments: Vec<String>, module_id: ModuleId) -> Self {
        Self {
            program,
            arguments,
            module_id,
        }
    }

    pub fn program(&self) -> &String {
        &self.program
    }

    pub fn arguments(&self) -> &Vec<String> {
        &self.arguments
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
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
        let (server, _source, notification_body_item_queue_receiver) = input.into();

        let mut deserializer = notification_body_item_queue_receiver.deserializer();
        let header = deserializer
            .deserialize::<ProcessDispatcherNotificationHeader>()
            .await
            .unwrap();

        let module_id = header.module_id();

        let mut child = Command::new(header.program)
            .args(header.arguments)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("running process");

        let (sender, receiver) = notification_body_item_queue();

        let mut stdout = child.stdout.take().expect("stdout handle");
        let mut stdin = child.stdin.take().expect("stdin handle");

        spawn(async move {
            loop {
                let mut buffer = Vec::with_capacity(4096);

                let result = stdout.read_buf(&mut buffer).await;

                if matches!(result, Ok(0)) || matches!(result, Err(_)){
                    break;
                }

                sender.do_send(Bytes::from(buffer)).await;
            }
        });

        spawn(async move {
            server.notify(module_id, receiver).await;
        });

        while let Some(item) = deserializer.next().await {
            stdin.write(&item).await.expect("successful write");
        }
    }
}
