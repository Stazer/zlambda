use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use zlambda_core::common::future::StreamExt;
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::server::{
    ServerId, ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize)]
pub struct RoundRobinNotificationEnvelope {
    module_id: ModuleId,
}

impl From<RoundRobinNotificationEnvelope> for (ModuleId,) {
    fn from(envelope: RoundRobinNotificationEnvelope) -> Self {
        (envelope.module_id,)
    }
}

impl RoundRobinNotificationEnvelope {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct RoundRobinScheduler {
    next_server_id: AtomicUsize,
}

#[async_trait::async_trait]
impl Module for RoundRobinScheduler {}

#[async_trait::async_trait]
impl ServerModule for RoundRobinScheduler {
    async fn on_notification(
        &self,
        mut input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let server_id = self.next_server_id.fetch_add(1, Ordering::Relaxed);

        let envelope_body = match input.body_mut().next().await {
            None => return,
            Some(envelope_body) => envelope_body,
        };
    }
}
