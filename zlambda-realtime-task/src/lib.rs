#![feature(if_let_guard)]

////////////////////////////////////////////////////////////////////////////////////////////////////

mod id;
mod instance;
mod log_entry_data;
mod manager;
mod message;
mod notification_header;
mod scheduling_task;
mod state;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use id::*;
pub use instance::*;
pub use log_entry_data::*;
pub use manager::*;
pub use message::*;
pub use notification_header::*;
pub use scheduling_task::*;
pub use state::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use zlambda_core::common::module::ModuleId;
use zlambda_core::common::notification::NotificationId;
use zlambda_core::server::{ServerClientId, ServerId};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTaskOrigin {
    server_id: ServerId,
    server_client_id: ServerClientId,
}

impl RealTimeTaskOrigin {
    pub fn new(server_id: ServerId, server_client_id: ServerClientId) -> Self {
        Self {
            server_id,
            server_client_id,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn server_client_id(&self) -> ServerClientId {
        self.server_client_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTask {
    id: RealTimeTaskId,
    state: RealTimeTaskState,
    target_module_id: ModuleId,
    source_server_id: ServerId,
    source_notification_id: NotificationId,
    deadline: Option<DateTime<Utc>>,
    origin: Option<RealTimeTaskOrigin>,
}

impl RealTimeTask {
    pub fn new(
        id: RealTimeTaskId,
        state: RealTimeTaskState,
        target_module_id: ModuleId,
        source_server_id: ServerId,
        source_notification_id: NotificationId,
        deadline: Option<DateTime<Utc>>,
        origin: Option<RealTimeTaskOrigin>,
    ) -> Self {
        Self {
            id,
            state,
            target_module_id,
            source_server_id,
            source_notification_id,
            deadline,
            origin,
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

    pub fn target_server_id(&self) -> Option<ServerId> {
        match &self.state {
            RealTimeTaskState::Dispatched(_state) => None,
            RealTimeTaskState::Scheduled(state) => Some(state.target_server_id()),
            RealTimeTaskState::Running(state) => Some(state.target_server_id()),
            RealTimeTaskState::Finished(state) => Some(state.target_server_id()),
        }
    }

    pub fn origin(&self) -> &Option<RealTimeTaskOrigin> {
        &self.origin
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DeadlineSortableRealTimeTask {
    task_id: RealTimeTaskId,
    deadline: Option<DateTime<Utc>>,
}

impl Eq for DeadlineSortableRealTimeTask {}

impl Ord for DeadlineSortableRealTimeTask {
    fn cmp(&self, other: &Self) -> Ordering {
        <Option<DateTime<Utc>> as Ord>::cmp(&self.deadline, &other.deadline)
    }
}

impl PartialEq for DeadlineSortableRealTimeTask {
    fn eq(&self, left: &Self) -> bool {
        <Option<DateTime<Utc>> as PartialEq>::eq(&self.deadline, &left.deadline)
    }
}

impl PartialOrd for DeadlineSortableRealTimeTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        <Option<DateTime<Utc>> as PartialOrd>::partial_cmp(&self.deadline, &other.deadline)
    }

    fn lt(&self, other: &Self) -> bool {
        <Option<DateTime<Utc>> as PartialOrd>::lt(&self.deadline, &other.deadline)
    }

    fn le(&self, other: &Self) -> bool {
        <Option<DateTime<Utc>> as PartialOrd>::le(&self.deadline, &other.deadline)
    }

    fn gt(&self, other: &Self) -> bool {
        <Option<DateTime<Utc>> as PartialOrd>::gt(&self.deadline, &other.deadline)
    }

    fn ge(&self, other: &Self) -> bool {
        <Option<DateTime<Utc>> as PartialOrd>::ge(&self.deadline, &other.deadline())
    }
}

impl DeadlineSortableRealTimeTask {
    pub fn new(task_id: RealTimeTaskId, deadline: Option<DateTime<Utc>>) -> Self {
        Self { task_id, deadline }
    }

    pub fn task_id(&self) -> RealTimeTaskId {
        self.task_id
    }

    pub fn deadline(&self) -> &Option<DateTime<Utc>> {
        &self.deadline
    }
}
