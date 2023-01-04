use zlambda_common::log::{LogEntryData, LogEntryId};
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type FollowerLogEntry = LogEntryData;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct FollowerLog {
    log_entries: Vec<Option<FollowerLogEntry>>,
    last_committed_log_entry_id: Option<LogEntryId>,
}

impl FollowerLog {
    pub fn get(&self, id: LogEntryId) -> Option<&FollowerLogEntry> {
        match self.log_entries.get(id) {
            Some(None) | None => None,
            Some(Some(entry)) => Some(entry),
        }
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }

    pub fn append(&mut self, log_entry_data: LogEntryData) {
        let log_entry_id = log_entry_data.id();
        let minimum_length = log_entry_id + 1;

        if self.log_entries.len() < minimum_length {
            self.log_entries.resize(minimum_length, None);
        }

        *self.log_entries.get_mut(log_entry_id).expect("to exist") = Some(log_entry_data);
    }

    pub fn commit(&mut self, log_entry_id: LogEntryId, term: Term) -> Vec<LogEntryId> {
        let mut missing_log_entry_ids = Vec::default();

        for log_entry_id in self.last_committed_log_entry_id.unwrap_or_default()..log_entry_id + 1 {
            match self.log_entries.get(log_entry_id) {
                Some(None) | None => missing_log_entry_ids.push(log_entry_id),
                Some(&Some(ref log_entry)) if log_entry.term() < term => {
                    missing_log_entry_ids.push(log_entry.id())
                }
                _ => {}
            }
        }

        missing_log_entry_ids
    }
}
