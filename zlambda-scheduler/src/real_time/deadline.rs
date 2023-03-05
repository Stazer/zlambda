use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use zlambda_core::common::async_trait;
use zlambda_core::common::deserialize::deserialize_from_bytes;
use zlambda_core::common::module::Module;
use zlambda_core::common::notification::{
    NotificationBodyItemQueueReceiver, NotificationBodyItemStreamExt,
};
use zlambda_core::common::serialize::serialize_to_bytes;
use zlambda_core::common::sync::Mutex;
use zlambda_core::server::{
    LogId, ServerId, ServerModule, ServerModuleCommitEventInput, ServerModuleCommitEventOutput,
    ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
    ServerSystemLogEntryData,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
struct Schedule {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct DeadlineRealTimeSchedulerNotificationHeader {
    deadline: DateTime<Utc>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub enum DeadlineRealTimeSchedulerLogEntryData {
    Register {
        server_id: ServerId,
        local_counter: usize,
        deadline: DateTime<Utc>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct DeadlineRealTimeScheduler {
    local_counter: AtomicUsize,
    receivers: Mutex<HashMap<usize, NotificationBodyItemQueueReceiver>>,
    schedule: Mutex<Schedule>,
}

#[async_trait]
impl Module for DeadlineRealTimeScheduler {}

#[async_trait]
impl ServerModule for DeadlineRealTimeScheduler {
    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (server, _source, notification_body_item_queue_receiver) = input.into();

        let local_counter = self.local_counter.fetch_add(1, Ordering::Relaxed);

        let mut deserializer = notification_body_item_queue_receiver.deserializer();

        let header = deserializer
            .deserialize::<DeadlineRealTimeSchedulerNotificationHeader>()
            .await
            .expect("deserialized header");

        server
        .commit(
            serialize_to_bytes(&DeadlineRealTimeSchedulerLogEntryData::Register {
                local_counter,
                server_id: server.server_id().await,
                deadline: header.deadline,
            })
            .unwrap()
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

        println!("HELLO WORLD");

        let bytes = match deserialize_from_bytes(log_entry.data()).expect("").0 {
            ServerSystemLogEntryData::Data(bytes) => bytes,
            _ => return,
        };

        let log_entry_data = deserialize_from_bytes::<DeadlineRealTimeSchedulerLogEntryData>(&bytes)
            .expect("derialized log entry data")
            .0;

        println!("{:?}", log_entry_data);
    }
}
