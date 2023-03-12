use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use zlambda_core::common::module::ModuleId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTaskManagerNotificationHeader {
    target_module_id: ModuleId,
    deadline: Option<DateTime<Utc>>,
    duration: Option<Duration>,
}

impl From<RealTimeTaskManagerNotificationHeader>
    for (ModuleId, Option<DateTime<Utc>>, Option<Duration>)
{
    fn from(header: RealTimeTaskManagerNotificationHeader) -> Self {
        (header.target_module_id, header.deadline, header.duration)
    }
}

impl RealTimeTaskManagerNotificationHeader {
    pub fn new(
        target_module_id: ModuleId,
        deadline: Option<DateTime<Utc>>,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            target_module_id,
            deadline,
            duration,
        }
    }

    pub fn target_module_id(&self) -> ModuleId {
        self.target_module_id
    }

    pub fn deadline(&self) -> &Option<DateTime<Utc>> {
        &self.deadline
    }

    pub fn duration(&self) -> &Option<Duration> {
        &self.duration
    }
}
