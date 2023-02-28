use serde::{Deserialize, Serialize};
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::{NotificationBodyItemQueueReceiver, NotificationBodyItemStreamExt};
use std::collections::HashMap;
use zlambda_core::common::serialize::serialize_to_bytes;
use zlambda_core::common::sync::Mutex;
use zlambda_core::server::{
    ServerId, ServerModule, ServerModuleCommitEventInput, ServerModuleCommitEventOutput,
    ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalRoundRobinSchedulerNotificationHeader {
    module_id: ModuleId,
}

impl From<GlobalRoundRobinSchedulerNotificationHeader> for (ModuleId,) {
    fn from(envelope: GlobalRoundRobinSchedulerNotificationHeader) -> Self {
        (envelope.module_id,)
    }
}

impl GlobalRoundRobinSchedulerNotificationHeader {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalRoundRobinLogEntryData {
    issuer_server_id: ServerId,
    local_counter: usize,
}

impl GlobalRoundRobinLogEntryData {
    pub fn new(issuer_server_id: ServerId, local_counter: usize) -> Self {
        Self {
            issuer_server_id,
            local_counter,
        }
    }

    pub fn issuer_server_id(&self) -> ServerId {
        self.issuer_server_id
    }

    pub fn local_counter(&self) -> usize {
        self.local_counter
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct GlobalRoundRobinScheduler {
    global_counter: Mutex<usize>,
    local_counter: Mutex<usize>,
    receivers: Mutex<HashMap<usize, NotificationBodyItemQueueReceiver>>,
}

#[async_trait::async_trait]
impl Module for GlobalRoundRobinScheduler {}

#[async_trait::async_trait]
impl ServerModule for GlobalRoundRobinScheduler {
    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (server, _source, notification_body_item_queue_receiver) = input.into();

        let local_counter = {
            let mut local_counter = self.local_counter.lock().await;
            *local_counter = *local_counter + 1;

            *local_counter
        };

        server
            .commit(
                serialize_to_bytes(
                    &GlobalRoundRobinLogEntryData::new(
                    server.server_id().await,
                        local_counter,
                ))
                    .expect("")
                    .into(),
            )
            .await;
    }

    async fn on_commit(
        &self,
        input: ServerModuleCommitEventInput,
    ) -> ServerModuleCommitEventOutput {
        let (server, log_entry_id) = input.into();
    }
}
