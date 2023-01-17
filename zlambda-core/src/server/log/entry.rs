use crate::server::{LogEntryData, LogTerm};
use serde::{Deserialize, Serialize};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LogEntryId = usize;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct LogEntry {
    id: LogEntryId,
    term: LogTerm,
    data: LogEntryData,
}

impl LogEntry {
    pub fn new(id: LogEntryId, term: LogTerm, data: LogEntryData) -> Self {
        Self { id, term, data }
    }

    pub fn id(&self) -> LogEntryId {
        self.id
    }

    pub fn term(&self) -> LogTerm {
        self.term
    }

    pub fn data(&self) -> &LogEntryData {
        &self.data
    }
}
