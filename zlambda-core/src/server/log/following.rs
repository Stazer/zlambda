use crate::server::{LogEntry, LogEntryId, LogTerm};

///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct FollowingLog {
    current_term: LogTerm,
    entries: Vec<Option<LogEntry>>,
    next_committing_log_entry_id: LogEntryId,
}

impl FollowingLog {
    pub fn new(
        current_term: LogTerm,
        entries: Vec<Option<LogEntry>>,
        next_committing_log_entry_id: LogEntryId,
    ) -> Self {
        Self {
            current_term,
            entries,
            next_committing_log_entry_id,
        }
    }

    pub fn current_term(&self) -> LogTerm {
        self.current_term
    }

    pub fn entries(&self) -> &Vec<Option<LogEntry>> {
        &self.entries
    }

    pub fn next_committing_log_entry_id(&self) -> LogEntryId {
        self.next_committing_log_entry_id
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        match self.next_committing_log_entry_id {
            0 => None,
            next_committing_log_entry_id => Some(next_committing_log_entry_id - 1),
        }
    }

    pub fn push(&mut self, log_entry: LogEntry) {
        let id = log_entry.id();

        if self.entries.len() < id + 1 {
            self.entries.resize_with(id + 1, || None);
        }

        *self.entries.get_mut(id).expect("valid entry") = Some(log_entry);
    }

    pub fn commit(&mut self, log_entry_id: LogEntryId, term: LogTerm) -> Vec<LogEntryId> {
        let mut missing_log_entry_ids = Vec::default();

        for log_entry_id in self.next_committing_log_entry_id..log_entry_id + 1 {
            match self.entries.get_mut(log_entry_id) {
                None | Some(None) => {
                    missing_log_entry_ids.push(log_entry_id);
                }
                Some(Some(log_entry)) if log_entry.term() < term => {
                    missing_log_entry_ids.push(log_entry_id);
                }
                Some(Some(_)) => {
                    if self.next_committing_log_entry_id == log_entry_id {
                        println!("{}", self.next_committing_log_entry_id);
                        self.next_committing_log_entry_id += 1;
                    }
                }
            }
        }

        missing_log_entry_ids
    }
}
