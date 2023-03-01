use serde::{Deserialize, Serialize};
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::server::{
    ServerId, ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};
use zlambda_core::common::async_trait;
use std::sync::atomic::{Ordering, AtomicUsize};

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
    counter: AtomicUsize,
}

#[async_trait]
impl Module for LocalRoundRobinScheduler {}

#[async_trait]
impl ServerModule for LocalRoundRobinScheduler {
    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (server, _source, notification_body_item_queue_receiver) = input.into();

        let counter = self.counter.fetch_add(1, Ordering::Relaxed);

        let next_server_id = {
            let socket_addresses = server.servers().socket_addresses().await;

            let mut next_server_id = counter % socket_addresses.len();

            loop {
                if let Some(Some(_)) = socket_addresses.get(next_server_id) {
                    break;
                }

                next_server_id += 1;

                if next_server_id >= socket_addresses.len() {
                    next_server_id = 0;
                }
            }

            ServerId::from(next_server_id)
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
