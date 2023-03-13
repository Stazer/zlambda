use crate::{
    RealTimeSharedData, RealTimeTaskId, RealTimeTaskSchedulingTaskMessage,
    RealTimeTaskSchedulingTaskPauseMessage, RealTimeTaskSchedulingTaskRescheduleMessage,
    RealTimeTaskSchedulingTaskResumeMessage,
};
use std::sync::Arc;
use zlambda_core::common::message::{message_queue, MessageQueueReceiver, MessageQueueSender};
use zlambda_core::common::runtime::{select, spawn};
use zlambda_core::common::time::sleep;

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
                select!(
                    message = self.receiver.do_receive() => {
                        self.on_real_time_task_scheduling_task_message(message).await
                    }
                )
            }
            RealTimeTaskSchedulingTaskState::Running => {
                let entry = {
                    let mut deadline_dispatched_sorted_tasks = self
                        .shared_data
                        .deadline_dispatched_sorted_tasks()
                        .write()
                        .await;

                    match deadline_dispatched_sorted_tasks.pop() {
                        None => None,
                        Some(entry) => Some((entry.0.task().id(), *entry.0.task().deadline())),
                    }
                };

                match entry {
                    Some((task_id, Some(deadline))) => {
                        select!(
                            notify = sleep(deadline) => {
                                self.schedule(task_id).await
                            },
                            message = self.receiver.do_receive() => {
                                self.on_real_time_task_scheduling_task_message(message).await
                            }
                        )
                    }
                    Some((task_id, None)) => self.schedule(task_id).await,
                    None => {
                        select!(
                            message = self.receiver.do_receive() => {
                                self.on_real_time_task_scheduling_task_message(message).await
                            }
                        )
                    }
                }
            }
        }
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
        println!("{}", task_id);
    }
}
