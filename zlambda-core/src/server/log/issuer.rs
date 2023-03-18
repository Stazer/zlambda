use crate::common::module::ModuleId;
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct LogModuleIssuer {
    module_id: ModuleId,
}

impl LogModuleIssuer {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct LogSystemIssuer {}

impl LogSystemIssuer {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Deserialize, Debug, Serialize)]
pub enum LogIssuer {
    Module(LogModuleIssuer),
    System(LogSystemIssuer),
}

impl From<LogModuleIssuer> for LogIssuer {
    fn from(issuer: LogModuleIssuer) -> Self {
        Self::Module(issuer)
    }
}

impl From<LogSystemIssuer> for LogIssuer {
    fn from(issuer: LogSystemIssuer) -> Self {
        Self::System(issuer)
    }
}
