use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use zlambda_core::common::future::StreamExt;
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::{
    NotificationBodyItemSinkExt, NotificationBodyItemStreamExt,
};
use zlambda_core::server::{
    ServerId, ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
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
        let server_id = self.next_server_id.fetch_add(1, Ordering::Relaxed);

        let mut reader = input.notification_body_item_queue_receiver_mut().reader();
        let header = reader
            .deserialize::<RoundRobinNotificationHeader>()
            .await
            .unwrap();

        println!("{} {:?}", server_id, header);
    }
}
