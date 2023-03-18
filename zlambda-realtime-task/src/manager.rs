use crate::{
    DeadlineSortableRealTimeTask, RealTimeTask, RealTimeTaskDispatchedState, RealTimeTaskId,
    RealTimeTaskManagerInstance, RealTimeTaskManagerLogEntryData,
    RealTimeTaskManagerLogEntryDispatchData, RealTimeTaskManagerNotificationHeader,
    RealTimeTaskRunningState, RealTimeTaskScheduledState, RealTimeTaskSchedulingTask,
    RealTimeTaskSchedulingTaskRescheduleMessageInput, RealTimeTaskSchedulingTaskState,
    RealTimeTaskState,
};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use zlambda_core::common::async_trait;
use zlambda_core::common::deserialize::deserialize_from_bytes;
use zlambda_core::common::module::Module;
use zlambda_core::common::notification::{NotificationBodyItemStreamExt, NotificationId};
use zlambda_core::common::serialize::serialize_to_bytes;
use zlambda_core::common::sync::RwLock;
use zlambda_core::server::{
    LogIssuer, LogModuleIssuer, ServerId, ServerModule, ServerModuleCommitEventInput,
    ServerModuleCommitEventOutput, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventOutput, ServerModuleStartupEventInput,
    ServerModuleStartupEventOutput, ServerSystemLogEntryData, SERVER_SYSTEM_LOG_ID,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct RealTimeTaskManager {
    instances: RwLock<HashMap<ServerId, Arc<RealTimeTaskManagerInstance>>>,
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
            let log_id = input
                .server()
                .logs()
                .create(Some(LogModuleIssuer::new(input.module_id()).into()))
                .await;

            let task = RealTimeTaskSchedulingTask::new(
                RealTimeTaskSchedulingTaskState::Running,
                log_id,
                input.server().clone(),
            );

            {
                let mut instances = self.instances.write().await;
                instances.insert(server_id, task.instance().clone());
            }

            task.spawn();
        }
    }

    async fn on_commit(
        &self,
        input: ServerModuleCommitEventInput,
    ) -> ServerModuleCommitEventOutput {
        if input.log_id() == SERVER_SYSTEM_LOG_ID {
            let log_entry = input
                .server()
                .logs()
                .get(input.log_id())
                .entries()
                .get(input.log_entry_id())
                .await
                .expect("");

            let data = deserialize_from_bytes::<ServerSystemLogEntryData>(log_entry.data())
                .expect("")
                .0;

            if let ServerSystemLogEntryData::CreateLog(data) = data {
                let issuer = LogIssuer::Module(LogModuleIssuer::new(input.module_id()));

                if matches!(data.log_issuer(), Some(issuer)) {
                    let task = RealTimeTaskSchedulingTask::new(
                        RealTimeTaskSchedulingTaskState::Paused,
                        data.log_id(),
                        input.server().clone(),
                    );

                    {
                        let mut instances = self.instances.write().await;
                        instances.insert(input.server().server_id().await, task.instance().clone());
                    }

                    task.spawn();
                }
            }
        } else {
            let server_id = input.server().server_id().await;

            let instance = {
                let instances = self.instances.read().await;
                instances.get(&server_id).expect("").clone()
            };

            if instance.log_id() != input.log_id() {
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
                    let (task_id, deadline) = {
                        let mut tasks = instance.tasks().write().await;

                        let task = RealTimeTask::new(
                            RealTimeTaskId::from(tasks.len()),
                            RealTimeTaskState::Dispatched(RealTimeTaskDispatchedState::new()),
                            data.target_module_id(),
                            data.source_server_id(),
                            data.source_notification_id(),
                            *data.deadline(),
                        );

                        let task_data = (task.id(), *task.deadline());

                        tasks.push(task);

                        task_data
                    };

                    {
                        let mut deadline_dispatched_sorted_tasks =
                            instance.deadline_sorted_tasks().write().await;

                        deadline_dispatched_sorted_tasks.push(Reverse(
                            DeadlineSortableRealTimeTask::new(task_id, deadline),
                        ));
                    }

                    instance
                        .sender()
                        .do_send_asynchronous(
                            RealTimeTaskSchedulingTaskRescheduleMessageInput::new(),
                        )
                        .await;
                }
                RealTimeTaskManagerLogEntryData::Schedule(data) => {
                    {
                        let mut tasks = instance.tasks().write().await;
                        let task = tasks.get_mut(usize::from(data.task_id())).expect("");
                        task.set_state(RealTimeTaskState::Scheduled(
                            RealTimeTaskScheduledState::new(data.target_server_id()),
                        ));
                    }

                    {
                        let mut deadline_sorted_tasks = instance.deadline_sorted_tasks().write().await;
                        deadline_sorted_tasks.retain(|Reverse(entry)| entry.task_id() != data.task_id());
                    }
                }
                RealTimeTaskManagerLogEntryData::Run(data) => {}
                RealTimeTaskManagerLogEntryData::Finish(data) => {}
            }
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

        let server_id = server.server_id().await;

        let instance = {
            let instances = self.instances.read().await;
            instances.get(&server_id).expect("").clone()
        };

        let notification_id =
            NotificationId::from(instance.local_counter().fetch_add(1, Ordering::Relaxed));

        {
            let mut receivers = instance.receivers().write().await;
            receivers.insert(notification_id, deserializer);
        }

        server
            .logs()
            .get(instance.log_id())
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
