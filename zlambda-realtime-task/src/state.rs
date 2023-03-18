use serde::{Deserialize, Serialize};
use zlambda_core::server::ServerId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct RealTimeTaskDispatchedState {}

impl From<RealTimeTaskDispatchedState> for () {
    fn from(_state: RealTimeTaskDispatchedState) -> Self {}
}

impl RealTimeTaskDispatchedState {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct RealTimeTaskScheduledState {
    target_server_id: ServerId,
}

impl From<RealTimeTaskScheduledState> for (ServerId,) {
    fn from(state: RealTimeTaskScheduledState) -> Self {
        (state.target_server_id,)
    }
}

impl RealTimeTaskScheduledState {
    pub fn new(target_server_id: ServerId) -> Self {
        Self { target_server_id }
    }

    pub fn target_server_id(&self) -> ServerId {
        self.target_server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct RealTimeTaskRunningState {}

impl From<RealTimeTaskRunningState> for () {
    fn from(_state: RealTimeTaskRunningState) -> Self {}
}

impl RealTimeTaskRunningState {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct RealTimeTaskFinishedState {}

impl From<RealTimeTaskFinishedState> for () {
    fn from(_state: RealTimeTaskFinishedState) -> Self {}
}

impl RealTimeTaskFinishedState {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub enum RealTimeTaskState {
    Dispatched(RealTimeTaskDispatchedState),
    Scheduled(RealTimeTaskScheduledState),
    Running(RealTimeTaskRunningState),
    Finished(RealTimeTaskFinishedState),
}

impl From<RealTimeTaskDispatchedState> for RealTimeTaskState {
    fn from(state: RealTimeTaskDispatchedState) -> Self {
        Self::Dispatched(state)
    }
}

impl From<RealTimeTaskScheduledState> for RealTimeTaskState {
    fn from(state: RealTimeTaskScheduledState) -> Self {
        Self::Scheduled(state)
    }
}

impl From<RealTimeTaskRunningState> for RealTimeTaskState {
    fn from(state: RealTimeTaskRunningState) -> Self {
        Self::Running(state)
    }
}

impl From<RealTimeTaskFinishedState> for RealTimeTaskState {
    fn from(state: RealTimeTaskFinishedState) -> Self {
        Self::Finished(state)
    }
}
