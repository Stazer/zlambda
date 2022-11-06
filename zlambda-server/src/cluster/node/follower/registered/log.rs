use crate::cluster::{LogEntryId, LogEntryType};
use std::cmp::max;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RegisteredFollowerNodeLogEntry {
    id: LogEntryId,
    r#type: LogEntryType,
}

impl RegisteredFollowerNodeLogEntry {
    pub fn new(id: LogEntryId, r#type: LogEntryType) -> Self {
        Self { id, r#type }
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
pub struct RegisteredFollowerNodeLogEntries {
    log_entries: HashMap<LogEntryId, RegisteredFollowerNodeLogEntry>,
    last_committed_log_entry_id: Option<LogEntryId>,
}

impl RegisteredFollowerNodeLogEntries {
    pub fn get(&self, id: LogEntryId) -> Option<&RegisteredFollowerNodeLogEntry> {
        self.log_entries.get(&id)
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }

    pub fn append(&mut self, log_entry_id: LogEntryId, r#type: LogEntryType) {
        self.log_entries.insert(
            log_entry_id,
            RegisteredFollowerNodeLogEntry::new(log_entry_id, r#type),
        );
    }

    pub fn commit(&mut self, log_entry_id: LogEntryId) {
        self.last_committed_log_entry_id = Some(max(
            log_entry_id,
            self.last_committed_log_entry_id.unwrap_or_default(),
        ));
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;
}
