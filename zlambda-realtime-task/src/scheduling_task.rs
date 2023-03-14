use crate::{
    RealTimeSharedData, RealTimeTaskId, RealTimeTaskSchedulingTaskMessage,
    RealTimeTaskSchedulingTaskPauseMessage, RealTimeTaskSchedulingTaskRescheduleMessage,
    RealTimeTaskSchedulingTaskResumeMessage, DeadlineDispatchedSortableRealTimeTask,
};
use std::sync::Arc;
use zlambda_core::common::message::{message_queue, MessageQueueReceiver, MessageQueueSender};
use zlambda_core::common::runtime::{select, spawn};
use zlambda_core::common::time::sleep;
use std::cmp::Reverse;
use chrono::Utc;

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
    shared_data: Arc<RealTimeSharedData>,
}

impl RealTimeTaskSchedulingTask {
    pub fn new(
        state: RealTimeTaskSchedulingTaskState,
        shared_data: Arc<RealTimeSharedData>,
    ) -> Self {
        let (sender, receiver) = message_queue();

        Self {
            sender,
            receiver,
            state,
            shared_data,
        }
    }

    pub fn sender(&self) -> &MessageQueueSender<RealTimeTaskSchedulingTaskMessage> {
        &self.sender
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
            RealTimeTaskSchedulingTaskState::Paused => {
                self.select_receiver().await
            }
            RealTimeTaskSchedulingTaskState::Running => {
                let entry = {
                    let mut deadline_dispatched_sorted_tasks = self
                        .shared_data
                        .deadline_dispatched_sorted_tasks()
                        .write()
                        .await;

                    deadline_dispatched_sorted_tasks.pop().map(|Reverse(entry)| entry)
                };

                let duration = (|| {
                    let entry = match &entry {
                        None => return None,
                        Some(entry) => entry,
                    };

                    let deadline = match *entry.task().deadline() {
                        None => return None,
                        Some(deadline) => deadline,
                    };

                    let duration = deadline - Utc::now();

                    if duration.is_zero() {
                        return None
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
                                self.schedule(entry.task().id()).await
                            },
                            message = self.receiver.do_receive() => {
                                {
                                    let mut deadline_dispatched_sorted_tasks = self
                                        .shared_data
                                        .deadline_dispatched_sorted_tasks()
                                        .write()
                                        .await;

                                    deadline_dispatched_sorted_tasks.push(Reverse(entry));
                                }

                                self.on_real_time_task_scheduling_task_message(message).await;
                            }
                        )
                    }
                    Some(entry) => {
                        self.schedule(entry.task().id()).await
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

    async fn reattach_task(&self, task: DeadlineDispatchedSortableRealTimeTask) {

    }

    async fn schedule(&self, task_id: RealTimeTaskId) {
        println!("{}", task_id);
    }
}
