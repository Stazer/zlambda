use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::{self, Reverse, Eq, Ord, PartialEq, PartialOrd};
use std::collections::{BinaryHeap, HashMap};
use std::sync::atomic::{self, AtomicUsize};
use std::sync::Arc;
use zlambda_core::common::async_trait;
use zlambda_core::common::deserialize::deserialize_from_bytes;
use zlambda_core::common::module::Module;
use zlambda_core::common::notification::{
    NotificationBodyItemQueueReceiver, NotificationBodyItemStreamExt,
    NotificationBodyStreamDeserializer, NotificationId,
};
use zlambda_core::common::serialize::serialize_to_bytes;
use zlambda_core::common::sync::Mutex;
use zlambda_core::common::utility::TaggedType;
use zlambda_core::server::{
    LogId, ServerId, ServerModule, ServerModuleCommitEventInput, ServerModuleCommitEventOutput,
    ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
    ServerSystemLogEntryData,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DeadlineRealTimeScheduleEntryData {
    server_id: ServerId,
    notification_id: NotificationId,
    deadline: DateTime<Utc>,
}

impl DeadlineRealTimeScheduleEntryData {
    pub fn new(
        server_id: ServerId,
        notification_id: NotificationId,
        deadline: DateTime<Utc>,
    ) -> Self {
        Self {
            server_id,
            notification_id,
            deadline,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn notification_id(&self) -> NotificationId {
        self.notification_id
    }

    pub fn deadline(&self) -> &DateTime<Utc> {
        &self.deadline
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DeadlineRealTimeScheduleEntryTag;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type DeadlineRealTimeScheduleEntryId = TaggedType<usize, DeadlineRealTimeScheduleEntryTag>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DeadlineRealTimeScheduleEntry {
    id: DeadlineRealTimeScheduleEntryId,
    data: DeadlineRealTimeScheduleEntryData,
}

impl DeadlineRealTimeScheduleEntry {
    pub fn new(
        id: DeadlineRealTimeScheduleEntryId,
        data: DeadlineRealTimeScheduleEntryData,
    ) -> Self {
        Self { id, data }
    }

    pub fn id(&self) -> DeadlineRealTimeScheduleEntryId {
        self.id
    }

    pub fn data(&self) -> &DeadlineRealTimeScheduleEntryData {
        &self.data
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DeadlineRealTimeScheduleDeadlineSortedEntry {
    entry: Arc<DeadlineRealTimeScheduleEntry>,
}

impl Eq for DeadlineRealTimeScheduleDeadlineSortedEntry {}

impl Ord for DeadlineRealTimeScheduleDeadlineSortedEntry {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        <DateTime<Utc> as Ord>::cmp(&self.entry.data.deadline, &other.entry.data.deadline)
    }
}

impl PartialEq for DeadlineRealTimeScheduleDeadlineSortedEntry {
    fn eq(&self, left: &Self) -> bool {
        <DateTime<Utc> as PartialEq>::eq(&self.entry.data.deadline, &left.entry.data.deadline)
    }
}

impl PartialOrd for DeadlineRealTimeScheduleDeadlineSortedEntry {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        <DateTime<Utc> as PartialOrd>::partial_cmp(
            &self.entry.data.deadline,
            &other.entry.data.deadline,
        )
    }

    fn lt(&self, other: &Self) -> bool {
        <DateTime<Utc> as PartialOrd>::lt(&self.entry.data.deadline, &other.entry.data.deadline)
    }

    fn le(&self, other: &Self) -> bool {
        <DateTime<Utc> as PartialOrd>::le(&self.entry.data.deadline, &other.entry.data.deadline)
    }

    fn gt(&self, other: &Self) -> bool {
        <DateTime<Utc> as PartialOrd>::gt(&self.entry.data.deadline, &other.entry.data.deadline)
    }

    fn ge(&self, other: &Self) -> bool {
        <DateTime<Utc> as PartialOrd>::ge(&self.entry.data.deadline, &other.entry.data.deadline)
    }
}

impl DeadlineRealTimeScheduleDeadlineSortedEntry {
    pub fn new(entry: Arc<DeadlineRealTimeScheduleEntry>) -> Self {
        Self { entry }
    }

    pub fn entry(&self) -> &Arc<DeadlineRealTimeScheduleEntry> {
        &self.entry
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct DeadlineRealTimeSchedule {
    entries: Vec<Arc<DeadlineRealTimeScheduleEntry>>,
    deadline_sorted_entries: BinaryHeap<Reverse<DeadlineRealTimeScheduleDeadlineSortedEntry>>,
}

impl DeadlineRealTimeSchedule {
    pub fn add(&mut self, data: DeadlineRealTimeScheduleEntryData) {
        let id = self.entries.len();

        let entry = Arc::new(DeadlineRealTimeScheduleEntry::new(
            DeadlineRealTimeScheduleEntryId::from(id),
            data,
        ));
        self.entries.push(entry.clone());
        self.deadline_sorted_entries
            .push(Reverse(DeadlineRealTimeScheduleDeadlineSortedEntry::new(entry)));
    }
}

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
        notification_id: NotificationId,
        deadline: DateTime<Utc>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct DeadlineRealTimeScheduler {
    local_counter: AtomicUsize,
    receivers: Mutex<
        HashMap<
            NotificationId,
            NotificationBodyStreamDeserializer<NotificationBodyItemQueueReceiver>,
        >,
    >,
    schedule: Mutex<DeadlineRealTimeSchedule>,
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

        let local_counter = self.local_counter.fetch_add(1, atomic::Ordering::Relaxed);

        let mut deserializer = notification_body_item_queue_receiver.deserializer();

        let header = deserializer
            .deserialize::<DeadlineRealTimeSchedulerNotificationHeader>()
            .await
            .expect("deserialized header");

        {
            let mut receivers = self.receivers.lock().await;
            receivers.insert(NotificationId::from(local_counter), deserializer);
        }

        server
            .commit(
                serialize_to_bytes(&ServerSystemLogEntryData::Data(
                    serialize_to_bytes(&DeadlineRealTimeSchedulerLogEntryData::Register {
                        notification_id: NotificationId::from(local_counter),
                        server_id: server.server_id().await,
                        deadline: header.deadline,
                    })
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

        let log_entry_data =
            deserialize_from_bytes::<DeadlineRealTimeSchedulerLogEntryData>(&bytes)
                .expect("derialized log entry data")
                .0;

        match log_entry_data {
            DeadlineRealTimeSchedulerLogEntryData::Register {
                server_id,
                notification_id,
                deadline,
            } => {
                let mut schedule = self.schedule.lock().await;

                schedule.add(DeadlineRealTimeScheduleEntryData::new(
                    server_id,
                    notification_id,
                    deadline,
                ));

               println!("{:?}", schedule);
            }
        }

        if server.server_id().await == server.leader_server_id().await {
        }
    }
}
