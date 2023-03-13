use serde::{Deserialize, Serialize};
use std::time::Duration;
use zlambda_core::common::module::ModuleId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct RealTimeTaskManagerNotificationHeader {
    target_module_id: ModuleId,
    deadline: Option<Duration>,
}

impl From<RealTimeTaskManagerNotificationHeader>
    for (ModuleId, Option<Duration>)
{
    fn from(header: RealTimeTaskManagerNotificationHeader) -> Self {
        (header.target_module_id, header.deadline)
    }
}

impl RealTimeTaskManagerNotificationHeader {
    pub fn new(
        target_module_id: ModuleId,
        deadline: Option<Duration>,
    ) -> Self {
        Self {
            target_module_id,
            deadline,
        }
    }

    pub fn target_module_id(&self) -> ModuleId {
        self.target_module_id
    }

    pub fn deadline(&self) -> &Option<Duration> {
        &self.deadline
    }
}
