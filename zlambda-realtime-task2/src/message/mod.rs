mod input;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use input::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use zlambda_core::common::message::AsynchronousMessage;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type RealTimeTaskSchedulingTaskPauseMessage =
    AsynchronousMessage<RealTimeTaskSchedulingTaskPauseMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type RealTimeTaskSchedulingTaskResumeMessage =
    AsynchronousMessage<RealTimeTaskSchedulingTaskResumeMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type RealTimeTaskSchedulingTaskRescheduleMessage =
    AsynchronousMessage<RealTimeTaskSchedulingTaskRescheduleMessageInput>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum RealTimeTaskSchedulingTaskMessage {
    Pause(RealTimeTaskSchedulingTaskPauseMessage),
    Resume(RealTimeTaskSchedulingTaskResumeMessage),
    Reschedule(RealTimeTaskSchedulingTaskRescheduleMessage),
}

impl From<RealTimeTaskSchedulingTaskPauseMessage> for RealTimeTaskSchedulingTaskMessage {
    fn from(input: RealTimeTaskSchedulingTaskPauseMessage) -> Self {
        Self::Pause(input)
    }
}

impl From<RealTimeTaskSchedulingTaskResumeMessage> for RealTimeTaskSchedulingTaskMessage {
    fn from(input: RealTimeTaskSchedulingTaskResumeMessage) -> Self {
        Self::Resume(input)
    }
}

impl From<RealTimeTaskSchedulingTaskRescheduleMessage> for RealTimeTaskSchedulingTaskMessage {
    fn from(input: RealTimeTaskSchedulingTaskRescheduleMessage) -> Self {
        Self::Reschedule(input)
    }
}
