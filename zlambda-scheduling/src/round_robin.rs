use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RoundRobinNotificationHeader {
    module_id: ModuleId,
}

impl From<RoundRobinNotificationHeader> for (ModuleId,) {
    fn from(envelope: RoundRobinNotificationHeader) -> Self {
        (envelope.module_id,)
    }
}

impl RoundRobinNotificationHeader {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct RoundRobinSchedulingModule {
    next_server_id: AtomicUsize,
}

#[async_trait::async_trait]
impl Module for RoundRobinSchedulingModule {}

#[async_trait::async_trait]
impl ServerModule for RoundRobinSchedulingModule {
    async fn on_notification(
        &self,
        mut input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let server_id = self.next_server_id.load(Ordering::Relaxed);

        let mut reader = input.notification_body_item_queue_receiver_mut().reader();
        let header = reader
            .deserialize::<RoundRobinNotificationHeader>()
            .await
            .unwrap();

        let socket_addresses = input.server().servers().socket_addresses().await;

        for (socket_address_server_id, socket_address) in socket_addresses.iter().enumerate() {
        }

        println!("{} {:?}", server_id, header);
    }
}
