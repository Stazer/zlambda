use crate::server::{LogEntryData, LogTerm};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

////////////////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(
    Copy, Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct LogEntryId(usize);

impl Display for LogEntryId {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl From<usize> for LogEntryId {
    fn from(value: usize) -> Self {
        Self::new(value)
    }
}

impl From<LogEntryId> for usize {
    fn from(server_id: LogEntryId) -> Self {
        server_id.0
    }
}

impl FromStr for LogEntryId {
    type Err = <usize as FromStr>::Err;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        <usize as FromStr>::from_str(string).map(Self::from)
    }
}

impl LogEntryId {
    fn new(value: usize) -> Self {
        Self(value)
    }
}

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
