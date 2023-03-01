use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use zlambda_core::common::async_trait;
use zlambda_core::common::deserialize::deserialize_from_bytes;
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::{
    NotificationBodyItemQueueReceiver, NotificationBodyItemStreamExt,
};
use zlambda_core::common::serialize::serialize_to_bytes;
use zlambda_core::common::sync::Mutex;
use zlambda_core::server::{
    LogEntryData, LogId, ServerId, ServerModule, ServerModuleCommitEventInput,
    ServerModuleCommitEventOutput, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventOutput, ServerSystemLogEntryData,
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
    global_counter: AtomicUsize,
    local_counter: AtomicUsize,
    receivers: Mutex<HashMap<usize, NotificationBodyItemQueueReceiver>>,
}

#[async_trait]
impl Module for GlobalRoundRobinScheduler {}

#[async_trait]
impl ServerModule for GlobalRoundRobinScheduler {
    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (server, _source, notification_body_item_queue_receiver) = input.into();

        let local_counter = self.local_counter.fetch_add(1, Ordering::Relaxed);

        {
            let mut receivers = self.receivers.lock().await;
            receivers.insert(local_counter, notification_body_item_queue_receiver);
        }

        server
            .commit(
                serialize_to_bytes(&ServerSystemLogEntryData::Data(
                    serialize_to_bytes(&GlobalRoundRobinLogEntryData::new(
                        server.server_id().await,
                        local_counter,
                    ))
                    .expect("")
                    .into(),
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

        let log_entry = server
            .logs()
            .get(LogId::from(0))
            .get(log_entry_id)
            .await
            .expect("existing log entry");

        let bytes = match deserialize_from_bytes(log_entry.data()).expect("").0 {
            ServerSystemLogEntryData::Data(bytes) => bytes,
            _ => return,
        };

        let log_entry_data = deserialize_from_bytes::<GlobalRoundRobinLogEntryData>(&bytes)
            .expect("derialized log entry data")
            .0;

        let global_counter = self.global_counter.fetch_add(1, Ordering::Relaxed);

        if log_entry_data.issuer_server_id() != server.server_id().await {
            return;
        }

        let receiver = {
            let mut receivers = self.receivers.lock().await;
            receivers.remove(&log_entry_data.local_counter())
        };

        let next_server_id = {
            let socket_addresses = server.servers().socket_addresses().await;

            let mut next_server_id = global_counter % socket_addresses.len();

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

        let mut deserializer = receiver.expect("existing receiver").deserializer();
        let header = deserializer
            .deserialize::<GlobalRoundRobinSchedulerNotificationHeader>()
            .await
            .unwrap();

        if let Some(server) = server.servers().get(next_server_id).await {
            server.notify(header.module_id(), deserializer).await;
        }
    }
}
