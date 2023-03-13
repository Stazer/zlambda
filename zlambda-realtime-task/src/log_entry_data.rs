use crate::RealTimeTaskId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use zlambda_core::common::module::ModuleId;
use zlambda_core::common::notification::NotificationId;
use zlambda_core::server::ServerId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTaskManagerLogEntryDispatchData {
    target_module_id: ModuleId,
    source_server_id: ServerId,
    source_notification_id: NotificationId,
    deadline: Option<DateTime<Utc>>,
    duration: Option<Duration>,
}

impl From<RealTimeTaskManagerLogEntryDispatchData>
    for (
        ModuleId,
        ServerId,
        NotificationId,
        Option<DateTime<Utc>>,
        Option<Duration>,
    )
{
    fn from(data: RealTimeTaskManagerLogEntryDispatchData) -> Self {
        (
            data.target_module_id,
            data.source_server_id,
            data.source_notification_id,
            data.deadline,
            data.duration,
        )
    }
}

impl RealTimeTaskManagerLogEntryDispatchData {
    pub fn new(
        target_module_id: ModuleId,
        source_server_id: ServerId,
        source_notification_id: NotificationId,
        deadline: Option<DateTime<Utc>>,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            target_module_id,
            source_server_id,
            source_notification_id,
            deadline,
            duration,
        }
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

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTaskManagerLogEntryScheduleData {
    task_id: RealTimeTaskId,
    target_server_id: ServerId,
}

impl From<RealTimeTaskManagerLogEntryScheduleData> for (RealTimeTaskId, ServerId) {
    fn from(data: RealTimeTaskManagerLogEntryScheduleData) -> Self {
        (data.task_id, data.target_server_id)
    }
}

impl RealTimeTaskManagerLogEntryScheduleData {
    pub fn new(task_id: RealTimeTaskId, target_server_id: ServerId) -> Self {
        Self {
            task_id,
            target_server_id,
        }
    }

    pub fn task_id(&self) -> RealTimeTaskId {
        self.task_id
    }

    pub fn target_server_id(&self) -> ServerId {
        self.target_server_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTaskManagerLogEntryRunData {
    task_id: RealTimeTaskId,
}

impl From<RealTimeTaskManagerLogEntryRunData> for (RealTimeTaskId,) {
    fn from(data: RealTimeTaskManagerLogEntryRunData) -> Self {
        (data.task_id,)
    }
}

impl RealTimeTaskManagerLogEntryRunData {
    pub fn new(task_id: RealTimeTaskId) -> Self {
        Self { task_id }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTaskManagerLogEntryFinishData {
    task_id: RealTimeTaskId,
}

impl From<RealTimeTaskManagerLogEntryFinishData> for (RealTimeTaskId,) {
    fn from(data: RealTimeTaskManagerLogEntryFinishData) -> Self {
        (data.task_id,)
    }
}

impl RealTimeTaskManagerLogEntryFinishData {
    pub fn new(task_id: RealTimeTaskId) -> Self {
        Self { task_id }
    }

    pub fn task_id(&self) -> RealTimeTaskId {
        self.task_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub enum RealTimeTaskManagerLogEntryData {
    Dispatch(RealTimeTaskManagerLogEntryDispatchData),
    Schedule(RealTimeTaskManagerLogEntryScheduleData),
    Run(RealTimeTaskManagerLogEntryRunData),
    Finish(RealTimeTaskManagerLogEntryFinishData),
}

impl From<RealTimeTaskManagerLogEntryDispatchData> for RealTimeTaskManagerLogEntryData {
    fn from(data: RealTimeTaskManagerLogEntryDispatchData) -> Self {
        RealTimeTaskManagerLogEntryData::Dispatch(data)
    }
}

impl From<RealTimeTaskManagerLogEntryScheduleData> for RealTimeTaskManagerLogEntryData {
    fn from(data: RealTimeTaskManagerLogEntryScheduleData) -> Self {
        RealTimeTaskManagerLogEntryData::Schedule(data)
    }
}

impl From<RealTimeTaskManagerLogEntryRunData> for RealTimeTaskManagerLogEntryData {
    fn from(data: RealTimeTaskManagerLogEntryRunData) -> Self {
        RealTimeTaskManagerLogEntryData::Run(data)
    }
}

impl From<RealTimeTaskManagerLogEntryFinishData> for RealTimeTaskManagerLogEntryData {
    fn from(data: RealTimeTaskManagerLogEntryFinishData) -> Self {
        RealTimeTaskManagerLogEntryData::Finish(data)
    }
}
