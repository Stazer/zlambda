use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use zlambda_core::common::async_trait;
use zlambda_core::common::bytes::Bytes;
use zlambda_core::common::future::stream::StreamExt;
use zlambda_core::common::io::{AsyncReadExt, AsyncWriteExt};
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::{
    notification_body_item_queue, NotificationBodyItemStreamExt,
};
use zlambda_core::common::runtime::{select, spawn};
use zlambda_core::server::{
    ServerClientId, ServerModule, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventInputSource, ServerModuleNotificationEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ProcessDispatcherNotificationTargetHeader {
    LocalServer {
        server_module_id: ModuleId,
    },
    LocalClient {
        server_client_id: ServerClientId,
        client_module_id: ModuleId,
    },
    Source {
        source_module_id: ModuleId,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct ProcessDispatcherNotificationHeader {
    program: String,
    arguments: Vec<String>,
    #[serde(flatten)]
    target: ProcessDispatcherNotificationTargetHeader,
}

impl From<ProcessDispatcherNotificationHeader>
    for (
        String,
        Vec<String>,
        ProcessDispatcherNotificationTargetHeader,
    )
{
    fn from(header: ProcessDispatcherNotificationHeader) -> Self {
        (header.program, header.arguments, header.target)
    }
}

impl ProcessDispatcherNotificationHeader {
    pub fn new(
        program: String,
        arguments: Vec<String>,
        target: ProcessDispatcherNotificationTargetHeader,
    ) -> Self {
        Self {
            program,
            arguments,
            target,
        }
    }

    pub fn program(&self) -> &String {
        &self.program
    }

    pub fn arguments(&self) -> &Vec<String> {
        &self.arguments
    }

    pub fn target(&self) -> &ProcessDispatcherNotificationTargetHeader {
        &self.target
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
        let (server, source, notification_body_item_queue_receiver) = input.into();

        let mut deserializer = notification_body_item_queue_receiver.deserializer();
        let header = deserializer
            .deserialize::<ProcessDispatcherNotificationHeader>()
            .await
            .unwrap();

        let (program, arguments, target) = header.into();

        let mut child = Command::new(program)
            .args(arguments)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("running process");

        let (sender, receiver) = notification_body_item_queue();

        let mut stdout = child.stdout.take().expect("stdout handle");
        let mut stdin = child.stdin.take().expect("stdin handle");

        spawn(async move {
            match target {
                ProcessDispatcherNotificationTargetHeader::LocalServer { server_module_id } => {
                    server.notify(server_module_id, receiver, None).await;
                }
                ProcessDispatcherNotificationTargetHeader::LocalClient {
                    server_client_id,
                    client_module_id,
                } => {
                    if let Some(client) = server.local_clients().get(server_client_id).await {
                        client.notify(client_module_id, receiver).await;
                    }
                }
                ProcessDispatcherNotificationTargetHeader::Source { source_module_id } => {
                    match source {
                        ServerModuleNotificationEventInputSource::Server(server_source) => {
                            if let Some(origin) = server_source.origin() {
                                if let Some(origin_server) =
                                    server.servers().get(origin.server_id()).await
                                {
                                    origin_server
                                        .clients()
                                        .get(origin.server_client_id())
                                        .notify(source_module_id, receiver)
                                        .await;
                                }
                            }
                        }
                        ServerModuleNotificationEventInputSource::Client(client_source) => {
                            if let Some(client) = server
                                .local_clients()
                                .get(client_source.server_client_id())
                                .await
                            {
                                client.notify(source_module_id, receiver).await;
                            }
                        }
                    }
                }
            }
        });

        loop {
            let mut buffer = Vec::with_capacity(4096);

            select!(
                output = stdout.read_buf(&mut buffer) => {
                    if buffer.is_empty() {
                        break
                    }

                    output.expect("");

                    sender.do_send(Bytes::from(buffer)).await
                },
                item = deserializer.next() => {
                    match item {
                        None => continue,
                        Some(item) => stdin.write(&item).await.expect("successful write"),
                    };
                }
            )
        }

        child.wait().await.expect("ok");
    }
}
