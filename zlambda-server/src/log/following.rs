use zlambda_common::log::{LogEntryData, LogEntryId, LogEntryType};
use zlambda_common::node::NodeId;
use std::cmp::max;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type FollowingLogEntry = LogEntryData;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct FollowingLog {
    log_entries: HashMap<LogEntryId, FollowingLogEntry>,
    last_committed_log_entry_id: Option<LogEntryId>,
}

impl FollowingLog {
    pub fn get(&self, id: LogEntryId) -> Option<&FollowingLogEntry> {
        self.log_entries.get(&id)
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        self.last_committed_log_entry_id
    }

    pub fn append(&mut self, log_entry_data: LogEntryData) {
        self.log_entries.insert(log_entry_data.id(), log_entry_data);
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
