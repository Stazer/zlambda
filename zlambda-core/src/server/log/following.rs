use crate::server::ServerId;
use crate::server::{LogEntry, LogEntryId, LogError, LogTerm};
use std::collections::hash_map::RandomState;
use std::collections::hash_set::Difference;
use std::collections::HashSet;

///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct FollowingLog {
    current_term: LogTerm,
    entries: Vec<Option<LogEntry>>,
    last_committed_log_entry_id: Option<LogEntryId>,
}

impl FollowingLog {
    pub fn new(
        current_term: LogTerm,
        entries: Vec<Option<LogEntry>>,
        last_committed_log_entry_id: Option<LogEntryId>,
    ) -> Self {
        Self {
            current_term,
            entries,
            last_committed_log_entry_id,
        }
    }

    pub fn current_term(&self) -> LogTerm {
        self.current_term
    }

    pub fn entries(&self) -> &Vec<Option<LogEntry>> {
        &self.entries
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }
}
