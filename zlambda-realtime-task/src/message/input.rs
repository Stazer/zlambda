#[derive(Debug)]
pub struct RealTimeTaskSchedulingTaskPauseMessageInput {}

impl From<RealTimeTaskSchedulingTaskPauseMessageInput> for () {
    fn from(_input: RealTimeTaskSchedulingTaskPauseMessageInput) -> Self {}
}

impl RealTimeTaskSchedulingTaskPauseMessageInput {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RealTimeTaskSchedulingTaskResumeMessageInput {}

impl From<RealTimeTaskSchedulingTaskResumeMessageInput> for () {
    fn from(_input: RealTimeTaskSchedulingTaskResumeMessageInput) -> Self {}
}

impl RealTimeTaskSchedulingTaskResumeMessageInput {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RealTimeTaskSchedulingTaskRescheduleMessageInput {}

impl From<RealTimeTaskSchedulingTaskRescheduleMessageInput> for () {
    fn from(_input: RealTimeTaskSchedulingTaskRescheduleMessageInput) -> Self {}
}

impl RealTimeTaskSchedulingTaskRescheduleMessageInput {
    pub fn new() -> Self {
        Self {}
    }
}
