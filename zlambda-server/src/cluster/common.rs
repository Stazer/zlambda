use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::algorithm::next_key;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NodeId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ConnectionId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type TermId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LogEntryId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LogEntryType {
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogEntry {
    id: LogEntryId,
    r#type: LogEntryType,
}

impl LogEntry {
    pub fn new(id: LogEntryId, r#type: LogEntryType) -> Self {
        Self {
            id,
            r#type,
        }
    }

    pub fn id(&self) -> LogEntryId {
        self.id
    }

    pub fn r#type(&self) -> &LogEntryType {
        &self.r#type
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct LeaderLog {
    next_id: LogEntryId,
    uncommitted_entries: HashMap<LogEntryId, LogEntry>,
    committed_entries: HashMap<LogEntryId, LogEntry>,
}

impl LeaderLog {
    pub fn committed_entries(&self) -> &HashMap<LogEntryId, LogEntry> {
        &self.committed_entries
    }

    pub fn uncommitted_entries(&self) -> &HashMap<LogEntryId, LogEntry> {
        &self.uncommitted_entries
    }

    pub fn begin(&mut self, r#type: LogEntryType) -> LogEntryId {
        let id = self.next_id;
        self.next_id += 1;

        self.uncommitted_entries.insert(id, LogEntry {
            id,
            r#type,
        });

        id
    }

    pub fn commit(&mut self, id: LogEntryId) {
        let entry = match self.uncommitted_entries.remove(&id) {
            Some(entry) => entry,
            None => return,
        };

        self.committed_entries.insert(id, entry);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct FollowerLog {
    uncommitted_entries: HashMap<LogEntryId, LogEntry>,
    committed_entries: HashMap<LogEntryId, LogEntry>,
}

impl FollowerLog {
    pub fn committed_entries(&self) -> &HashMap<LogEntryId, LogEntry> {
        &self.committed_entries
    }

    pub fn uncommitted_entries(&self) -> &HashMap<LogEntryId, LogEntry> {
        &self.uncommitted_entries
    }

    pub fn begin(&mut self, entry: LogEntry) {
        self.uncommitted_entries.insert(entry.id(), entry);
    }

    pub fn commit(&mut self, id: LogEntryId) {

    }
}
