use crate::{
    RealTimeTaskSchedulingTaskMessage, RealTimeTaskSchedulingTaskPauseMessage,
    RealTimeTaskSchedulingTaskResumeMessage, RealTimeTaskSchedulingTaskRescheduleMessage
};
use zlambda_core::common::message::{message_queue, MessageQueueReceiver, MessageQueueSender};
use zlambda_core::common::runtime::{select, spawn};

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
}

impl RealTimeTaskSchedulingTask {
    pub fn new(state: RealTimeTaskSchedulingTaskState) -> Self {
        let (sender, receiver) = message_queue();

        Self {
            sender,
            receiver,
            state,
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
                select!(
                    message = self.receiver.do_receive() => {
                        self.on_real_time_task_scheduling_task_message(message).await
                    }
                )
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
}
