mod id;
mod log_entry_data;
mod manager;
mod message;
mod notification_header;
mod scheduling_task;
mod shared;
mod state;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use id::*;
pub use log_entry_data::*;
pub use manager::*;
pub use message::*;
pub use notification_header::*;
pub use scheduling_task::*;
pub use shared::*;
pub use state::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
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
    deadline: Option<Duration>,
}

impl RealTimeTask {
    pub fn new(
        id: RealTimeTaskId,
        state: RealTimeTaskState,
        target_module_id: ModuleId,
        source_server_id: ServerId,
        source_notification_id: NotificationId,
        deadline: Option<Duration>,
    ) -> Self {
        Self {
            id,
            state,
            target_module_id,
            source_server_id,
            source_notification_id,
            deadline,
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

    pub fn deadline(&self) -> &Option<Duration> {
        &self.deadline
    }

    pub fn target_server_id(&self) -> &Option<ServerId> {
        &None
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DeadlineDispatchedSortableRealTimeTask {
    task: Arc<RealTimeTask>,
}

impl Eq for DeadlineDispatchedSortableRealTimeTask {}

impl Ord for DeadlineDispatchedSortableRealTimeTask {
    fn cmp(&self, other: &Self) -> Ordering {
        match <RealTimeTaskState as Ord>::cmp(self.task.state(), other.task.state()) {
            Ordering::Equal => {
                <Option<Duration> as Ord>::cmp(self.task.deadline(), other.task.deadline())
            }
            ordering => ordering,
        }
    }
}

impl PartialEq for DeadlineDispatchedSortableRealTimeTask {
    fn eq(&self, left: &Self) -> bool {
        <RealTimeTaskState as PartialEq>::eq(self.task.state(), left.task.state())
            && <Option<Duration> as PartialEq>::eq(self.task.deadline(), left.task.deadline())
    }
}

impl PartialOrd for DeadlineDispatchedSortableRealTimeTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match <RealTimeTaskState as PartialOrd>::partial_cmp(self.task.state(), other.task.state())
        {
            Some(Ordering::Equal) => {}
            ordering => return ordering,
        };

        <Option<Duration> as PartialOrd>::partial_cmp(self.task.deadline(), other.task.deadline())
    }

    fn lt(&self, other: &Self) -> bool {
        <RealTimeTaskState as PartialOrd>::lt(self.task.state(), other.task.state())
            && <Option<Duration> as PartialOrd>::lt(self.task.deadline(), other.task.deadline())
    }

    fn le(&self, other: &Self) -> bool {
        <RealTimeTaskState as PartialOrd>::le(self.task.state(), other.task.state())
            && <Option<Duration> as PartialOrd>::le(self.task.deadline(), other.task.deadline())
    }

    fn gt(&self, other: &Self) -> bool {
        <RealTimeTaskState as PartialOrd>::gt(self.task.state(), other.task.state())
            && <Option<Duration> as PartialOrd>::gt(self.task.deadline(), other.task.deadline())
    }

    fn ge(&self, other: &Self) -> bool {
        <RealTimeTaskState as PartialOrd>::ge(self.task.state(), other.task.state())
            && <Option<Duration> as PartialOrd>::ge(self.task.deadline(), other.task.deadline())
    }
}

impl DeadlineDispatchedSortableRealTimeTask {
    pub fn new(task: Arc<RealTimeTask>) -> Self {
        Self { task }
    }

    pub fn task(&self) -> &Arc<RealTimeTask> {
        &self.task
    }
}
