use crate::RealTimeTaskId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use zlambda_core::common::module::ModuleId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTaskManagerDispatchNotificationHeader {
    target_module_id: ModuleId,
    deadline: Option<DateTime<Utc>>,
}

impl From<RealTimeTaskManagerDispatchNotificationHeader> for (ModuleId, Option<DateTime<Utc>>) {
    fn from(header: RealTimeTaskManagerDispatchNotificationHeader) -> Self {
        (header.target_module_id, header.deadline)
    }
}

impl RealTimeTaskManagerDispatchNotificationHeader {
    pub fn new(target_module_id: ModuleId, deadline: Option<DateTime<Utc>>) -> Self {
        Self {
            target_module_id,
            deadline,
        }
    }

    pub fn target_module_id(&self) -> ModuleId {
        self.target_module_id
    }

    pub fn deadline(&self) -> &Option<DateTime<Utc>> {
        &self.deadline
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTaskManagerExecuteNotificationHeader {
    task_id: RealTimeTaskId,
}

impl From<RealTimeTaskManagerExecuteNotificationHeader> for (RealTimeTaskId,) {
    fn from(header: RealTimeTaskManagerExecuteNotificationHeader) -> Self {
        (header.task_id,)
    }
}

impl RealTimeTaskManagerExecuteNotificationHeader {
    pub fn new(task_id: RealTimeTaskId) -> Self {
        Self { task_id }
    }

    pub fn task_id(&self) -> RealTimeTaskId {
        self.task_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
//#[serde(tag = "type", rename_all="lowercase")]
pub enum RealTimeTaskManagerNotificationHeader {
    Dispatch(RealTimeTaskManagerDispatchNotificationHeader),
    Execute(RealTimeTaskManagerExecuteNotificationHeader),
}

impl From<RealTimeTaskManagerDispatchNotificationHeader> for RealTimeTaskManagerNotificationHeader {
    fn from(header: RealTimeTaskManagerDispatchNotificationHeader) -> Self {
        Self::Dispatch(header)
    }
}

impl From<RealTimeTaskManagerExecuteNotificationHeader> for RealTimeTaskManagerNotificationHeader {
    fn from(header: RealTimeTaskManagerExecuteNotificationHeader) -> Self {
        Self::Execute(header)
    }
}
