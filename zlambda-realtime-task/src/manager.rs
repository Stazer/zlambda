use crate::{
    DeadlineDispatchedSortableRealTimeTask, RealTimeSharedData, RealTimeTask,
    RealTimeTaskDispatchedState, RealTimeTaskId, RealTimeTaskManagerLogEntryData,
    RealTimeTaskManagerLogEntryDispatchData, RealTimeTaskManagerNotificationHeader,
    RealTimeTaskSchedulingTask, RealTimeTaskSchedulingTaskRescheduleMessageInput, RealTimeTaskSchedulingTaskState,
    RealTimeTaskState,
};
use std::cmp::Reverse;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use zlambda_core::common::async_trait;
use zlambda_core::common::deserialize::deserialize_from_bytes;
use zlambda_core::common::module::Module;
use zlambda_core::common::notification::{NotificationBodyItemStreamExt, NotificationId};
use zlambda_core::common::serialize::serialize_to_bytes;
use zlambda_core::server::{
    ServerModule, ServerModuleCommitEventInput, ServerModuleCommitEventOutput,
    ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
    ServerModuleStartupEventInput, ServerModuleStartupEventOutput,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct RealTimeTaskManager {
    shared_data: Arc<RealTimeSharedData>,
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
        let leader_server_id = input.server().leader_server_id().await;

        if server_id == leader_server_id {
            let log_id = input.server().logs().create().await;

            {
                let mut log_ids = self.shared_data.log_ids().write().await;
                log_ids.insert(server_id, log_id);
            }
        }

        {
            let task = RealTimeTaskSchedulingTask::new(
                if server_id == leader_server_id {
                    RealTimeTaskSchedulingTaskState::Running
                } else {
                    RealTimeTaskSchedulingTaskState::Paused
                },
                self.shared_data.clone(),
            );

            let mut senders = self.shared_data.senders().write().await;
            senders.insert(server_id, task.sender().clone());

            task.spawn();
        }

        {
            let mut servers = self.shared_data.servers().write().await;
            servers.insert(server_id, input.server().clone());
        }
    }

    async fn on_commit(
        &self,
        input: ServerModuleCommitEventInput,
    ) -> ServerModuleCommitEventOutput {
        let server_id = input.server().server_id().await;

        let log_id = {
            let log_ids = self
                .shared_data
                .log_ids()
                .read()
                .await;

            match log_ids
                .get(&server_id) {
                Some(log_id) => *log_id,
                None => return,
            }
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
                    let mut tasks = self.shared_data.tasks().write().await;

                    let task = Arc::new(RealTimeTask::new(
                        RealTimeTaskId::from(tasks.len()),
                        RealTimeTaskState::Dispatched(RealTimeTaskDispatchedState::new()),
                        data.target_module_id(),
                        data.source_server_id(),
                        data.source_notification_id(),
                        *data.deadline(),
                    ));

                    tasks.push(task.clone());

                    task
                };

                {
                    let mut deadline_dispatched_sorted_tasks = self
                        .shared_data
                        .deadline_dispatched_sorted_tasks()
                        .write()
                        .await;

                    deadline_dispatched_sorted_tasks
                        .push(Reverse(DeadlineDispatchedSortableRealTimeTask::new(task)));
                }

                let sender = {
                    let senders = self.shared_data.senders().read().await;
                    senders.get(&server_id).expect("").clone()
                };

                sender.do_send_asynchronous(RealTimeTaskSchedulingTaskRescheduleMessageInput::new()).await;
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
            let log_ids = self.shared_data.log_ids().read().await;
            *log_ids.get(&server.server_id().await).expect("")
        };

        let notification_id = NotificationId::from(
            self.shared_data
                .local_counter()
                .fetch_add(1, Ordering::Relaxed),
        );

        {
            let mut receivers = self.shared_data.receivers().write().await;
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
                    ),
                ))
                .expect("")
                .freeze(),
            )
            .await;
    }
}

impl RealTimeTaskManager {}
