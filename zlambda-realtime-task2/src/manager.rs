use crate::{
    DeadlineSortableRealTimeTask, RealTimeTask, RealTimeTaskDispatchedState,
    RealTimeTaskFinishedState, RealTimeTaskId, RealTimeTaskManagerExecuteNotificationHeader,
    RealTimeTaskManagerInstance, RealTimeTaskManagerLogEntryData,
    RealTimeTaskManagerLogEntryDispatchData, RealTimeTaskManagerLogEntryFinishData,
    RealTimeTaskManagerLogEntryOriginData,
    RealTimeTaskManagerLogEntryRescheduleData, RealTimeTaskManagerLogEntryRunData,
    RealTimeTaskManagerNotificationHeader, RealTimeTaskOrigin, RealTimeTaskRunningState,
    RealTimeTaskScheduledState, RealTimeTaskSchedulingTask,
    RealTimeTaskSchedulingTaskRescheduleMessageInput, RealTimeTaskSchedulingTaskState,
    RealTimeTaskState,
};
use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use zlambda_core::common::async_trait;
use zlambda_core::common::bytes::BytesMut;
use zlambda_core::common::deserialize::deserialize_from_bytes;
use zlambda_core::common::future::stream::StreamExt;
use zlambda_core::common::module::Module;
use zlambda_core::common::notification::{
    notification_body_item_queue, NotificationBodyItemStreamExt, NotificationBodyItemType,
    NotificationId,
};
use zlambda_core::common::runtime::spawn;
use zlambda_core::common::serialize::serialize_to_bytes;
use zlambda_core::common::sync::RwLock;
use zlambda_core::common::tracing::debug;
use zlambda_core::server::{
    LogIssuer, LogModuleIssuer, ServerId, ServerModule, ServerModuleCommitEventInput,
    ServerModuleCommitEventOutput, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventInputServerSource,
    ServerModuleNotificationEventInputServerSourceOrigin, ServerModuleNotificationEventInputSource,
    ServerModuleNotificationEventOutput, ServerModuleServerConnectEventInput,
    ServerModuleServerConnectEventOutput, ServerModuleServerDisconnectEventInput,
    ServerModuleServerDisconnectEventOutput, ServerModuleStartupEventInput,
    ServerModuleStartupEventOutput, ServerNotificationOrigin, ServerSystemLogEntryData,
    SERVER_SYSTEM_LOG_ID,
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
                let _issuer = LogIssuer::Module(LogModuleIssuer::new(input.module_id()));
                let server_id = input.server().server_id().await;
                let leader_server_id = input.server().leader_server_id().await;

                if matches!(data.log_issuer(), Some(_issuer)) && server_id != leader_server_id {
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
                .entries()
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
                            data.origin().as_ref().map(|o| {
                                RealTimeTaskOrigin::new(o.server_id(), o.server_client_id())
                            }),
                            data.bytes().clone(),
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
                    let (
                        source_server_id,
                        origin,
                        target_module_id,
                        notification_data,
                    ) = {
                        let mut tasks = instance.tasks().write().await;
                        let task = tasks.get_mut(usize::from(data.task_id())).expect("");
                        task.set_state(RealTimeTaskState::Scheduled(
                            RealTimeTaskScheduledState::new(data.target_server_id()),
                        ));

                        (
                            task.source_server_id(),
                            task.origin().as_ref().map(|o| {
                                ServerNotificationOrigin::new(o.server_id(), o.server_client_id())
                            }),
                            task.target_module_id(),
                            task.notification().clone(),
                        )
                    };

                    {
                        let mut occupations = instance.occupations().write().await;
                        occupations
                            .entry(data.target_server_id())
                            .or_insert(HashSet::default())
                            .insert(data.task_id());
                    }

                    {
                        let mut deadline_sorted_tasks =
                            instance.deadline_sorted_tasks().write().await;
                        deadline_sorted_tasks
                            .retain(|Reverse(entry)| entry.task_id() != data.task_id());
                    }

                    if source_server_id == instance.server().server_id().await {
                        let (sender, receiver) = notification_body_item_queue();

                        let task_id = data.task_id();

                        spawn(async move {
                            let bytes = serialize_to_bytes(
                                &RealTimeTaskManagerNotificationHeader::Execute(
                                    RealTimeTaskManagerExecuteNotificationHeader::new(
                                        task_id,
                                        target_module_id,
                                    ),
                                ),
                            )
                            .expect("")
                            .freeze();

                            let data = serialize_to_bytes(&NotificationBodyItemType::Binary(bytes))
                                .expect("");

                            sender.do_send(data).await;
                            sender.do_send(notification_data).await;
                        });

                        spawn(async move {
                            instance
                                .server()
                                .servers()
                                .get(data.target_server_id())
                                .await
                                .expect("")
                                .notify(input.module_id(), receiver, origin)
                                .await;
                        });
                    }
                }
                RealTimeTaskManagerLogEntryData::Reschedule(data) => {
                    let (task_id, deadline) = {
                        let mut tasks = instance.tasks().write().await;
                        let task = tasks.get_mut(usize::from(data.task_id())).expect("");
                        task.set_state(RealTimeTaskState::Dispatched(
                            RealTimeTaskDispatchedState::new(),
                        ));

                        let task_data = (task.id(), *task.deadline());

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
                RealTimeTaskManagerLogEntryData::Run(data) => {
                    let mut tasks = instance.tasks().write().await;
                    let task = tasks.get_mut(usize::from(data.task_id())).expect("");
                    task.set_state(RealTimeTaskState::Running(RealTimeTaskRunningState::new(
                        task.target_server_id().expect(""),
                    )));
                }
                RealTimeTaskManagerLogEntryData::Finish(data) => {
                    let (target_server_id,) = {
                        let mut tasks = instance.tasks().write().await;
                        let task = tasks.get_mut(usize::from(data.task_id())).expect("");

                        let target_server_id = task.target_server_id().expect("");

                        task.set_state(RealTimeTaskState::Finished(
                            RealTimeTaskFinishedState::new(target_server_id),
                        ));

                        (target_server_id,)
                    };

                    {
                        let mut occupations = instance.occupations().write().await;
                        occupations
                            .entry(target_server_id)
                            .or_insert(HashSet::default())
                            .remove(&data.task_id());
                    }
                }
            }
        }
    }

    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (server, source, notification_body_item_queue_receiver) = input.into();

        let mut deserializer = notification_body_item_queue_receiver.deserializer();
        let server_id = server.server_id().await;

        let instance = {
            let instances = self.instances.read().await;
            instances.get(&server_id).expect("").clone()
        };

        let header = deserializer
            .deserialize::<RealTimeTaskManagerNotificationHeader>()
            .await
            .expect("");

        let origin = match source {
            ServerModuleNotificationEventInputSource::Server(server) => {
                if let Some(origin) = server.origin() {
                    Some(RealTimeTaskManagerLogEntryOriginData::new(
                        origin.server_id(),
                        origin.server_client_id(),
                    ))
                } else {
                    None
                }
            }
            ServerModuleNotificationEventInputSource::Client(client) => {
                Some(RealTimeTaskManagerLogEntryOriginData::new(
                    server.server_id().await,
                    client.server_client_id(),
                ))
            }
        };

        match header {
            RealTimeTaskManagerNotificationHeader::Dispatch(header) => {
                let notification_id =
                    NotificationId::from(instance.local_counter().fetch_add(1, Ordering::Relaxed));

                let mut bytes = BytesMut::default();

                while let Some(item) = deserializer.next().await {
                    bytes.extend(item);
                }

                server
                    .logs()
                    .get(instance.log_id())
                    .entries()
                    .commit(
                        serialize_to_bytes(&RealTimeTaskManagerLogEntryData::Dispatch(
                            RealTimeTaskManagerLogEntryDispatchData::new(
                                header.target_module_id(),
                                server.server_id().await,
                                notification_id,
                                *header.deadline(),
                                origin,
                                bytes.freeze(),
                            ),
                        ))
                        .expect("")
                        .freeze(),
                    )
                    .await;
            }
            RealTimeTaskManagerNotificationHeader::Execute(header) => {
                let module = instance
                    .server()
                    .modules()
                    .get(header.target_module_id())
                    .await
                    .expect("");

                let (sender, receiver) = notification_body_item_queue();

                server
                    .logs()
                    .get(instance.log_id())
                    .entries()
                    .commit(
                        serialize_to_bytes(&RealTimeTaskManagerLogEntryData::Run(
                            RealTimeTaskManagerLogEntryRunData::new(header.task_id()),
                        ))
                        .expect("")
                        .freeze(),
                    )
                    .await;

                spawn(async move {
                    while let Some(item) = deserializer.next().await {
                        sender.do_send(item).await;
                    }
                });

                module
                    .on_notification(ServerModuleNotificationEventInput::new(
                        server.clone(),
                        header.target_module_id(),
                        ServerModuleNotificationEventInputServerSource::new(
                            origin.as_ref().map(|o| {
                                ServerModuleNotificationEventInputServerSourceOrigin::new(
                                    o.server_id(),
                                    o.server_client_id(),
                                )
                            }),
                            instance.server().server_id().await,
                        )
                        .into(),
                        receiver,
                    ))
                    .await;

                server
                    .logs()
                    .get(instance.log_id())
                    .entries()
                    .commit(
                        serialize_to_bytes(&RealTimeTaskManagerLogEntryData::Finish(
                            RealTimeTaskManagerLogEntryFinishData::new(header.task_id()),
                        ))
                        .expect("")
                        .freeze(),
                    )
                    .await;
            }
        }
    }

    async fn on_server_connect(
        &self,
        input: ServerModuleServerConnectEventInput,
    ) -> ServerModuleServerConnectEventOutput {
        let current_server_id = input.server().server_id().await;
        let connected_server_id = input.server_id();

        let instance = {
            let instances = self.instances.read().await;
            instances.get(&current_server_id).expect("").clone()
        };

        {
            let mut disconnected_servers = instance.disconnected_servers().write().await;
            disconnected_servers.remove(&connected_server_id);
        }
    }

    async fn on_server_disconnect(
        &self,
        input: ServerModuleServerDisconnectEventInput,
    ) -> ServerModuleServerDisconnectEventOutput {
        let disconnected_server_id = input.server_id();
        let current_server_id = input.server().server_id().await;
        let leader_server_id = input.server().leader_server_id().await;

        if current_server_id != leader_server_id {
            return;
        }

        let instance = {
            let instances = self.instances.read().await;
            instances.get(&current_server_id).expect("").clone()
        };

        {
            let mut occupations = instance.occupations().write().await;
            if let Some(bucket) = occupations.get_mut(&disconnected_server_id) {
                bucket.clear();
            }
        }

        {
            let mut disconnected_servers = instance.disconnected_servers().write().await;
            disconnected_servers.insert(disconnected_server_id);
        }

        let rescheduling_task_ids = {
            let tasks = instance.tasks().read().await;

            tasks
                .iter()
                .filter(|task| match &task.state {
                    RealTimeTaskState::Dispatched(_) => false,
                    RealTimeTaskState::Scheduled(scheduled) => {
                        scheduled.target_server_id() == disconnected_server_id
                    }
                    RealTimeTaskState::Running(running) => {
                        running.target_server_id() == disconnected_server_id
                    }
                    RealTimeTaskState::Finished(_) => false,
                })
                .map(|task| task.id())
                .collect::<Vec<_>>()
        };

        debug!(
            "Reschedule tasks {:?} of disconnected server {}",
            &rescheduling_task_ids, disconnected_server_id
        );

        for rescheduling_task_id in rescheduling_task_ids {
            input
                .server()
                .logs()
                .get(instance.log_id())
                .entries()
                .commit(
                    serialize_to_bytes(&RealTimeTaskManagerLogEntryData::Reschedule(
                        RealTimeTaskManagerLogEntryRescheduleData::new(rescheduling_task_id),
                    ))
                    .expect("")
                    .freeze(),
                )
                .await;
        }
    }
}

impl RealTimeTaskManager {}
