use crate::server::ServerId;
use crate::server::{LogEntry, LogEntryId, LogError, LogTerm};
use std::collections::hash_map::RandomState;
use std::collections::hash_set::Difference;
use std::collections::HashSet;

///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct LogManager {
    current_term: LogTerm,
    entries: Vec<LogEntry>,
    acknowledgeable_server_ids: Vec<HashSet<ServerId>>,
    acknowledged_server_ids: Vec<HashSet<ServerId>>,
    last_committed_log_entry_id: Option<LogEntryId>,
}

impl LogManager {
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
            _ => return None,
        }
    }

    pub fn is_acknowledged(&self, id: LogEntryId) -> Option<bool> {
        match (
            self.quorum_count(id),
            self.remaining_acknowledgeable_server_ids(id),
        ) {
            (Some(quorum_count), Some(remaining_acknowledgeable_server_ids)) => {
                Some(remaining_acknowledgeable_server_ids.count() <= quorum_count)
            }
            _ => None,
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

        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
