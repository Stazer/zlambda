use crate::{
    DeadlineSortableRealTimeTask, RealTimeTaskId, RealTimeTaskManagerInstance,
    RealTimeTaskManagerLogEntryData, RealTimeTaskManagerLogEntryScheduleData,
    RealTimeTaskSchedulingTaskMessage, RealTimeTaskSchedulingTaskPauseMessage,
    RealTimeTaskSchedulingTaskRescheduleMessage, RealTimeTaskSchedulingTaskResumeMessage,
};
use chrono::Utc;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use zlambda_core::common::message::{message_queue, MessageQueueReceiver, MessageQueueSender};
use zlambda_core::common::runtime::{select, spawn};
use zlambda_core::common::serialize::serialize_to_bytes;
use zlambda_core::common::sync::RwLock;
use zlambda_core::common::time::sleep;
use zlambda_core::common::tracing::debug;
use zlambda_core::server::{LogId, Server};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum RealTimeTaskSchedulingTaskState {
    Paused,
    Running,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RealTimeTaskSchedulingTask {
    sender: MessageQueueSender<RealTimeTaskSchedulingTaskMessage>,
    receiver: MessageQueueReceiver<RealTimeTaskSchedulingTaskMessage>,
    state: RealTimeTaskSchedulingTaskState,
    instance: Arc<RealTimeTaskManagerInstance>,
}

impl RealTimeTaskSchedulingTask {
    pub fn new(state: RealTimeTaskSchedulingTaskState, log_id: LogId, server: Arc<Server>) -> Self {
        let (sender, receiver) = message_queue();

        Self {
            sender: sender.clone(),
            receiver,
            state,
            instance: Arc::new(RealTimeTaskManagerInstance::new(
                AtomicUsize::default(),
                log_id,
                RwLock::new(HashMap::default()),
                RwLock::new(Vec::default()),
                RwLock::new(BinaryHeap::default()),
                RwLock::new(HashMap::default()),
                sender,
                server,
            )),
        }
    }

    pub fn sender(&self) -> &MessageQueueSender<RealTimeTaskSchedulingTaskMessage> {
        &self.sender
    }

    pub fn instance(&self) -> &Arc<RealTimeTaskManagerInstance> {
        &self.instance
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
    }

    pub async fn run(mut self) {
        loop {
            self.select().await
        }
    }

    async fn select(&mut self) {
        match self.state {
            RealTimeTaskSchedulingTaskState::Paused => self.select_receiver().await,
            RealTimeTaskSchedulingTaskState::Running => {
                let entry = {
                    let mut deadline_dispatched_sorted_tasks =
                        self.instance.deadline_sorted_tasks().write().await;

                    deadline_dispatched_sorted_tasks
                        .pop()
                        .map(|Reverse(entry)| entry)
                };

                let duration = (|| {
                    let entry = match &entry {
                        None => return None,
                        Some(entry) => entry,
                    };

                    let deadline = match *entry.deadline() {
                        None => return None,
                        Some(deadline) => deadline,
                    };

                    let duration = deadline - Utc::now();

                    if duration.is_zero() {
                        return None;
                    }

                    match duration.to_std() {
                        Err(_error) => None,
                        Ok(duration) => Some(duration),
                    }
                })();

                match entry {
                    Some(entry) if let Some(duration) = duration => {
                        select!(
                            _ = sleep(duration) => {
                                self.schedule(entry.task_id()).await
                            },
                            message = self.receiver.do_receive() => {
                                {
                                    let mut deadline_dispatched_sorted_tasks = self
                                        .instance
                                        .deadline_sorted_tasks()
                                        .write()
                                        .await;

                                    deadline_dispatched_sorted_tasks.push(Reverse(entry));
                                }

                                self.on_real_time_task_scheduling_task_message(message).await;
                            }
                        )
                    }
                    Some(entry) => {
                        self.schedule(entry.task_id()).await
                    }
                    None => {
                        self.select_receiver().await
                    }
                }
            }
        }
    }

    async fn select_receiver(&mut self) {
        select!(
            message = self.receiver.do_receive() => {
                self.on_real_time_task_scheduling_task_message(message).await
            }
        )
    }

    async fn on_real_time_task_scheduling_task_message(
        &mut self,
        message: RealTimeTaskSchedulingTaskMessage,
    ) {
        match message {
            RealTimeTaskSchedulingTaskMessage::Pause(message) => {
                self.on_real_time_task_scheduling_task_pause_message(message)
                    .await
            }
            RealTimeTaskSchedulingTaskMessage::Resume(message) => {
                self.on_real_time_task_scheduling_task_resume_message(message)
                    .await
            }
            RealTimeTaskSchedulingTaskMessage::Reschedule(message) => {
                self.on_real_time_task_scheduling_task_reschedule_message(message)
                    .await
            }
        }
    }

    async fn on_real_time_task_scheduling_task_pause_message(
        &mut self,
        _message: RealTimeTaskSchedulingTaskPauseMessage,
    ) {
        self.state = RealTimeTaskSchedulingTaskState::Paused;
    }

    async fn on_real_time_task_scheduling_task_resume_message(
        &mut self,
        _message: RealTimeTaskSchedulingTaskResumeMessage,
    ) {
        self.state = RealTimeTaskSchedulingTaskState::Running;
    }

    async fn on_real_time_task_scheduling_task_reschedule_message(
        &mut self,
        _message: RealTimeTaskSchedulingTaskRescheduleMessage,
    ) {
    }

    async fn schedule(&self, task_id: RealTimeTaskId) {
        let minimum = {
            let occupations = self.instance.occupations().read().await;

            let mut minimum = None;

            //let server_ids = Vec::default();

            /*for server in self.instance.server().servers().iter().await {
                server_ids.push(server.server_id())
            }*/

            for (server_id, tasks) in occupations.iter() {
                minimum = match minimum {
                    None => Some((*server_id, tasks.len())),
                    Some((_server_id, tasks_length)) if tasks.len() < tasks_length => {
                        Some((*server_id, tasks.len()))
                    }
                    minimum => minimum,
                }
            }

            minimum
        };

        let (server_id, tasks_length) = match minimum {
            Some((server_id, tasks_length)) => (server_id, tasks_length),
            None => {
                let source_server_id = {
                    let tasks = self.instance.tasks().read().await;
                    tasks
                        .get(usize::from(task_id))
                        .expect("")
                        .source_server_id()
                };

                (source_server_id, 0)
            }
        };

        debug!("Schedule task {} to server {} with {} tasks", task_id, server_id, tasks_length);

        let data = serialize_to_bytes(&RealTimeTaskManagerLogEntryData::Schedule(
            RealTimeTaskManagerLogEntryScheduleData::new(task_id, server_id),
        ))
        .expect("")
        .freeze();

        self.instance
            .server()
            .logs()
            .get(self.instance.log_id())
            .entries()
            .commit(data)
            .await;
    }
}
