mod id;
mod log_entry_data;
mod manager;
mod notification_header;
mod state;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use id::*;
pub use log_entry_data::*;
pub use manager::*;
pub use notification_header::*;
pub use state::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::{self, Eq, Ord, PartialEq, PartialOrd};
use std::sync::Arc;
use std::time::Duration;
use zlambda_core::common::module::ModuleId;
use zlambda_core::common::notification::NotificationId;
use zlambda_core::server::ServerId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTask {
    id: RealTimeTaskId,
    state: RealTimeTaskState,
    target_module_id: ModuleId,
    source_server_id: ServerId,
    source_notification_id: NotificationId,
    deadline: Option<DateTime<Utc>>,
    duration: Option<Duration>,
}

impl RealTimeTask {
    pub fn new(
        id: RealTimeTaskId,
        state: RealTimeTaskState,
        target_module_id: ModuleId,
        source_server_id: ServerId,
        source_notification_id: NotificationId,
        deadline: Option<DateTime<Utc>>,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            id,
            state,
            target_module_id,
            source_server_id,
            source_notification_id,
            deadline,
            duration,
        }
    }

    pub fn id(&self) -> RealTimeTaskId {
        self.id
    }

    pub fn state(&self) -> &RealTimeTaskState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut RealTimeTaskState {
        &mut self.state
    }

    pub fn set_state(&mut self, state: RealTimeTaskState) {
        self.state = state
    }

    pub fn target_module_id(&self) -> ModuleId {
        self.target_module_id
    }

    pub fn source_server_id(&self) -> ServerId {
        self.source_server_id
    }

    pub fn source_notification_id(&self) -> NotificationId {
        self.source_notification_id
    }

    pub fn deadline(&self) -> &Option<DateTime<Utc>> {
        &self.deadline
    }

    pub fn duration(&self) -> &Option<Duration> {
        &self.duration
    }
}

/*/////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct SortableRealTimeTask<T> {
    comparator:
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DeadlineSortableRealTimeTask {
    task: Arc<RealTimeTask>,
}

impl Eq for DeadlineSortableRealTimeTask {}

impl Ord for DeadlineSortableRealTimeTask {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        <Option<DateTime<Utc>> as Ord>::cmp(self.task.deadline(), other.task.deadline())
    }
}

impl PartialEq for DeadlineSortableRealTimeTask {
    fn eq(&self, left: &Self) -> bool {
        <Option<DateTime<Utc>> as PartialEq>::eq(self.task.deadline(), left.task.deadline())
    }
}

impl PartialOrd for DeadlineSortableRealTimeTask {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        <Option<DateTime<Utc>> as PartialOrd>::partial_cmp(self.task.deadline(), other.task.deadline())
    }

    fn lt(&self, other: &Self) -> bool {
        <Option<DateTime<Utc>> as PartialOrd>::lt(self.task.deadline(), other.task.deadline())
    }

    fn le(&self, other: &Self) -> bool {
        <Option<DateTime<Utc>> as PartialOrd>::le(self.task.deadline(), other.task.deadline())
    }

    fn gt(&self, other: &Self) -> bool {
        <Option<DateTime<Utc>> as PartialOrd>::gt(self.task.deadline(), other.task.deadline())
    }

    fn ge(&self, other: &Self) -> bool {
        <Option<DateTime<Utc>> as PartialOrd>::ge(self.task.deadline(), other.task.deadline())
    }
}

impl DeadlineSortableRealTimeTask {
    pub fn new(task: Arc<RealTimeTask>) -> Self {
        Self { task }
    }

    pub fn task(&self) -> &Arc<RealTimeTask> {
        &self.task
    }
}*/
