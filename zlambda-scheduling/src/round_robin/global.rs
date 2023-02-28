use serde::{Deserialize, Serialize};
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::common::sync::Mutex;
use zlambda_core::common::serialize::serialize_to_bytes;
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
}

impl GlobalRoundRobinLogEntryData {
    pub fn new(
        issuer_server_id: ServerId,
    ) -> Self {
        Self {
            issuer_server_id,
        }
    }

    pub fn issuer_server_id(&self) -> ServerId {
        self.issuer_server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct GlobalRoundRobinScheduler {
    counter: Mutex<ServerId>,
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

        server.commit(serialize_to_bytes(&GlobalRoundRobinLogEntryData::new(
            server.server_id().await,
        )).expect("").into()).await;
    }

    async fn on_commit(
        &self,
        input: ServerModuleCommitEventInput,
    ) -> ServerModuleCommitEventOutput {
        let counter = {
            let mut counter = self.counter.lock().await;
            *counter = ServerId::from(usize::from(*counter) + 1);

            *counter
        };

        println!("{:?}", counter);
        println!("{:?}", input);
    }
}
