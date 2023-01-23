use crate::server::{LogEntry, LogEntryId, LogTerm};

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

    pub fn push(&mut self, log_entry: LogEntry) {
        if self.entries.len() < log_entry.id() {
            self.entries.resize(log_entry.id() - 1, None);
        }
    }

    pub fn commit(&mut self, log_entry_id: LogEntryId, term: LogTerm) -> Vec<LogEntryId> {
        let mut missing_log_entry_ids = Vec::default();

        for log_entry_id in self.last_committed_log_entry_id.unwrap_or_default()..log_entry_id + 1 {
            match self.entries.get(log_entry_id) {
                Some(None) | None => missing_log_entry_ids.push(log_entry_id),
                Some(Some(ref log_entry)) if log_entry.term() < term => {
                    missing_log_entry_ids.push(log_entry.id())
                }
                _ => {
                    if self.last_committed_log_entry_id.map(|x| x + 1) == Some(log_entry_id) {
                        self.last_committed_log_entry_id = Some(log_entry_id)
                    }
                }
            }
        }

        missing_log_entry_ids
    }
}
