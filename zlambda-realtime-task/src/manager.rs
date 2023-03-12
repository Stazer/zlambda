use crate::{
    DeadlineDispatchedSortableRealTimeTask, RealTimeTask, RealTimeTaskDispatchedState,
    RealTimeTaskId, RealTimeTaskManagerLogEntryData, RealTimeTaskManagerLogEntryDispatchData,
    RealTimeTaskManagerNotificationHeader, RealTimeTaskState,
};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use zlambda_core::common::async_trait;
use zlambda_core::common::deserialize::deserialize_from_bytes;
use zlambda_core::common::module::Module;
use zlambda_core::common::notification::{
    NotificationBodyItemQueueReceiver, NotificationBodyItemStreamExt,
    NotificationBodyStreamDeserializer, NotificationId,
};
use zlambda_core::common::runtime::spawn;
use zlambda_core::common::serialize::serialize_to_bytes;
use zlambda_core::common::sync::{Mutex, RwLock};
use zlambda_core::common::task::JoinHandle;
use zlambda_core::server::{
    Server,
    LogId, ServerId, ServerModule, ServerModuleCommitEventInput, ServerModuleCommitEventOutput,
    ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
    ServerModuleStartupEventInput, ServerModuleStartupEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct RealTimeTaskManager {
    local_counter: AtomicUsize,
    log_ids: RwLock<HashMap<ServerId, LogId>>,
    receivers: RwLock<
        HashMap<
            NotificationId,
            NotificationBodyStreamDeserializer<NotificationBodyItemQueueReceiver>,
        >,
    >,
    tasks: RwLock<Vec<Arc<RealTimeTask>>>,
    scheduling_handle: Mutex<Option<JoinHandle<()>>>,
    deadline_dispatched_sorted_tasks:
        Arc<RwLock<BinaryHeap<Reverse<DeadlineDispatchedSortableRealTimeTask>>>>,
}

#[async_trait]
impl Module for RealTimeTaskManager {}

#[async_trait]
impl ServerModule for RealTimeTaskManager {
    async fn on_startup(
        &self,
        input: ServerModuleStartupEventInput,
    ) -> ServerModuleStartupEventOutput {
        let server_id = input.server().server_id().await;

        if server_id == input.server().server_id().await {
            let log_id = input.server().logs().create().await;

            {
                let mut log_ids = self.log_ids.write().await;
                log_ids.insert(server_id, log_id);
            }
        }
    }

    async fn on_commit(
        &self,
        input: ServerModuleCommitEventInput,
    ) -> ServerModuleCommitEventOutput {
        let log_id = {
            *self
                .log_ids
                .read()
                .await
                .get(&input.server().server_id().await)
                .expect("")
        };

        if log_id != input.log_id() {
            return;
        }

        let log_entry = input
            .server()
            .logs()
            .get(input.log_id())
            .get(input.log_entry_id())
            .await
            .expect("");

        let data = deserialize_from_bytes::<RealTimeTaskManagerLogEntryData>(log_entry.data())
            .expect("")
            .0;

        match data {
            RealTimeTaskManagerLogEntryData::Dispatch(data) => {
                let task = {
                    let mut tasks = self.tasks.write().await;

                    let task = Arc::new(RealTimeTask::new(
                        RealTimeTaskId::from(tasks.len()),
                        RealTimeTaskState::Dispatched(RealTimeTaskDispatchedState::new()),
                        data.target_module_id(),
                        data.source_server_id(),
                        data.source_notification_id(),
                        *data.deadline(),
                        *data.duration(),
                    ));

                    tasks.push(task.clone());

                    task
                };

                {
                    let mut deadline_dispatched_sorted_tasks =
                        self.deadline_dispatched_sorted_tasks.write().await;

                    deadline_dispatched_sorted_tasks
                        .push(Reverse(DeadlineDispatchedSortableRealTimeTask::new(task)));
                }

                self.spawn_scheduling_task(input.server()).await;
            }
            RealTimeTaskManagerLogEntryData::Schedule(data) => {}
            RealTimeTaskManagerLogEntryData::Run(data) => {}
            RealTimeTaskManagerLogEntryData::Finish(data) => {}
        }
    }

    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (server, _source, notification_body_item_queue_receiver) = input.into();

        let mut deserializer = notification_body_item_queue_receiver.deserializer();
        let header = deserializer
            .deserialize::<RealTimeTaskManagerNotificationHeader>()
            .await
            .unwrap();

        let log_id = {
            let log_ids = self.log_ids.read().await;
            *log_ids.get(&server.server_id().await).expect("")
        };

        let notification_id =
            NotificationId::from(self.local_counter.fetch_add(1, Ordering::Relaxed));

        {
            let mut receivers = self.receivers.write().await;
            receivers.insert(notification_id, deserializer);
        }

        server
            .logs()
            .get(log_id)
            .commit(
                serialize_to_bytes(&RealTimeTaskManagerLogEntryData::Dispatch(
                    RealTimeTaskManagerLogEntryDispatchData::new(
                        header.target_module_id(),
                        server.server_id().await,
                        notification_id,
                        *header.deadline(),
                        *header.duration(),
                    ),
                ))
                .expect("")
                .freeze(),
            )
            .await;
    }
}

impl RealTimeTaskManager {
    async fn spawn_scheduling_task(&self, server: &Arc<Server>) {
        let mut handle = self.scheduling_handle.lock().await;

        if handle.is_none() {
            *handle = Some(spawn(async move {}));
        }
    }
}
