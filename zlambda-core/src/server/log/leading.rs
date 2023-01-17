use crate::server::ServerId;
use crate::server::{LogEntry, LogEntryData, LogEntryId, LogError, LogTerm};
use std::collections::hash_map::RandomState;
use std::collections::hash_set::Difference;
use std::collections::HashSet;

///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct LeadingLog {
    current_term: LogTerm,
    entries: Vec<LogEntry>,
    acknowledgeable_server_ids: Vec<HashSet<ServerId>>,
    acknowledged_server_ids: Vec<HashSet<ServerId>>,
    next_committing_log_entry_id: LogEntryId,
}

impl LeadingLog {
    pub fn current_term(&self) -> LogTerm {
        self.current_term
    }

    pub fn entries(&self) -> &Vec<LogEntry> {
        &self.entries
    }

    pub fn quorum_count(&self, id: LogEntryId) -> Option<usize> {
        self.acknowledgeable_server_ids
            .get(id)
            .map(|server_ids| server_ids.len() / 2)
    }



    pub fn acknowledgeable_server_ids(&self, id: LogEntryId) -> Option<&HashSet<ServerId>> {
        self.acknowledgeable_server_ids.get(id)
    }

    pub fn acknowledged_server_ids(&self, id: LogEntryId) -> Option<&HashSet<ServerId>> {
        self.acknowledged_server_ids.get(id)
    }

    pub fn remaining_acknowledgeable_server_ids(
        &self,
        id: LogEntryId,
    ) -> Option<Difference<'_, ServerId, RandomState>> {
        match (
            self.acknowledgeable_server_ids(id),
            self.acknowledged_server_ids(id),
        ) {
            (Some(acknowledgeable_server_ids), Some(acknowledged_server_ids)) => {
                Some(acknowledgeable_server_ids.difference(acknowledged_server_ids))
            }
            _ => None,
        }
    }

    pub fn next_committing_log_entry_id(&self) -> LogEntryId {
        self.next_committing_log_entry_id
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        match self.next_committing_log_entry_id {
            0 => None,
            next_committing_log_entry_id => Some(next_committing_log_entry_id - 1)
        }
    }

    pub fn is_acknowledged(&self, id: LogEntryId) -> bool {
        match (
            self.quorum_count(id),
            self.remaining_acknowledgeable_server_ids(id),
        ) {
            (Some(quorum_count), Some(remaining_acknowledgeable_server_ids)) => {
                remaining_acknowledgeable_server_ids.count() <= quorum_count
            }
            _ => false,
        }
    }

    pub fn is_committed(&self, id: LogEntryId) -> bool {
        match self.last_committed_log_entry_id() {
            None => false,
            Some(last_committed_log_entry_id) => last_committed_log_entry_id <= id
        }
    }

    pub fn acknowledge(&mut self, id: LogEntryId, server_id: ServerId) -> Result<(), LogError> {
        let server_ids = match self.acknowledgeable_server_ids.get(id) {
            Some(server_ids) => server_ids,
            None => return Err(LogError::NotExisting),
        };

        if !server_ids.contains(&server_id) {
            return Err(LogError::NotAcknowledgeable);
        }

        let server_ids = match self.acknowledged_server_ids.get_mut(id) {
            Some(server_ids) => server_ids,
            None => return Err(LogError::NotExisting),
        };

        if server_ids.contains(&server_id) {
            return Err(LogError::AlreadyAcknowledged);
        }

        server_ids.insert(server_id);

        loop {
            if self.is_committed(self.next_committing_log_entry_id) {
                self.next_committing_log_entry_id += 1;
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn append(&mut self, log_entries_data: Vec<LogEntryData>) -> Vec<LogEntryId> {
        self.entries.reserve(log_entries_data.len());
        let start = self.entries.len() - 1;

        self.entries.extend(
            log_entries_data
                .into_iter()
                .enumerate()
                .map(|(index, data)| LogEntry::new(start + index, self.current_term, data)),
        );

        (start..self.entries.len()).collect()
    }
}
