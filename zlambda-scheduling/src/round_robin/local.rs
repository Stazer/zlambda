use serde::{Deserialize, Serialize};
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::common::sync::Mutex;
use zlambda_core::server::{
    ServerId, ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalRoundRobinSchedulerNotificationHeader {
    module_id: ModuleId,
}

impl From<LocalRoundRobinSchedulerNotificationHeader> for (ModuleId,) {
    fn from(envelope: LocalRoundRobinSchedulerNotificationHeader) -> Self {
        (envelope.module_id,)
    }
}

impl LocalRoundRobinSchedulerNotificationHeader {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct LocalRoundRobinScheduler {
    next_server_id: Mutex<ServerId>,
}

#[async_trait::async_trait]
impl Module for LocalRoundRobinScheduler{}

#[async_trait::async_trait]
impl ServerModule for LocalRoundRobinScheduler {
    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (server, _source, notification_body_item_queue_receiver) = input.into();

        let next_server_id = {
            let socket_addresses = server.servers().socket_addresses().await;

            let mut next_server_id = self.next_server_id.lock().await;

            match socket_addresses
                .iter()
                .enumerate()
                .map(|(server_id, _)| ServerId::from(server_id))
                .filter(|server_id| server_id > &next_server_id)
                .next()
            {
                Some(server_id) => {
                    *next_server_id = server_id;
                }
                None => {
                    if let Some(server_id) = socket_addresses
                        .iter()
                        .enumerate()
                        .map(|(server_id, _)| ServerId::from(server_id))
                        .next()
                    {
                        *next_server_id = server_id;
                    }
                }
            }

            *next_server_id
        };

        let mut deserializer = notification_body_item_queue_receiver.deserializer();
        let header = deserializer
            .deserialize::<LocalRoundRobinSchedulerNotificationHeader>()
            .await
            .unwrap();

        if let Some(server) = server.servers().get(next_server_id).await {
            server.notify(header.module_id(), deserializer).await;
        }
    }
}
