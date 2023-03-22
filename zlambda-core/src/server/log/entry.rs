use crate::common::bytes::Bytes;
use crate::common::utility::TaggedType;
use crate::server::{LogTerm, ServerId};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LogEntryIssueIdTag;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LogEntryIssueId = TaggedType<usize, LogEntryIssueIdTag>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogEntryServerIssuer {
    server_id: ServerId,
    issue_id: LogEntryIssueId,
}

impl LogEntryServerIssuer {
    pub fn new(server_id: ServerId, issue_id: LogEntryIssueId) -> Self {
        Self {
            server_id,
            issue_id,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn issue_id(&self) -> LogEntryIssueId {
        self.issue_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LogEntryIssuer {
    Server(LogEntryServerIssuer),
}

impl From<LogEntryServerIssuer> for LogEntryIssuer {
    fn from(issuer: LogEntryServerIssuer) -> Self {
        Self::Server(issuer)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LogEntryData = Bytes;

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogEntry {
    id: LogEntryId,
    term: LogTerm,
    data: LogEntryData,
    issuer: Option<LogEntryIssuer>,
}

impl LogEntry {
    pub fn new(
        id: LogEntryId,
        term: LogTerm,
        data: LogEntryData,
        issuer: Option<LogEntryIssuer>,
    ) -> Self {
        Self {
            id,
            term,
            data,
            issuer,
        }
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

    pub fn issuer(&self) -> &Option<LogEntryIssuer> {
        &self.issuer
    }
}
