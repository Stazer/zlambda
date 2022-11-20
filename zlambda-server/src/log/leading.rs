use crate::algorithm::next_key;
use std::collections::hash_map::RandomState;
use std::collections::hash_set::Difference;
use std::collections::{HashMap, HashSet};
use zlambda_common::log::{LogEntryData, LogEntryId, LogEntryType};
use zlambda_common::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeadingLogEntry {
    data: LogEntryData,
    acknowledging_nodes: HashSet<NodeId>,
    acknowledged_nodes: HashSet<NodeId>,
}

impl LeadingLogEntry {
    pub fn new(
        data: LogEntryData,
        acknowledging_nodes: HashSet<NodeId>,
        acknowledged_nodes: HashSet<NodeId>,
    ) -> Self {
        Self {
            data,
            acknowledging_nodes,
            acknowledged_nodes,
        }
    }

    pub fn data(&self) -> &LogEntryData {
        &self.data
    }

    pub fn acknowledging_nodes(&self) -> &HashSet<NodeId> {
        &self.acknowledging_nodes
    }

    pub fn acknowledged_nodes(&self) -> &HashSet<NodeId> {
        &self.acknowledged_nodes
    }

    pub fn remaining_acknowledging_nodes(&self) -> Difference<'_, NodeId, RandomState> {
        self.acknowledging_nodes
            .difference(&self.acknowledged_nodes)
    }

    pub fn quorum_count(&self) -> usize {
        self.acknowledging_nodes.len() / 2
    }

    pub fn acknowledge(&mut self, node_id: NodeId) {
        if !self.acknowledging_nodes.contains(&node_id) {
            panic!(
                "Log entry {} cannot be acknowledged by node {}",
                self.data.id(),
                node_id
            );
        }

        if self.acknowledged_nodes.contains(&node_id) {
            panic!(
                "Log entry {} already acknowledged by node {}",
                self.data.id(),
                node_id
            );
        }

        self.acknowledged_nodes.insert(node_id);
    }

    pub fn is_committed(&self) -> bool {
        self.remaining_acknowledging_nodes().count() <= self.quorum_count()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct LeadingLog {
    log_entries: HashMap<LogEntryId, LeadingLogEntry>,
    next_committing_log_entry_id: LogEntryId,
}

impl LeadingLog {
    fn next_log_entry_id(&self) -> LogEntryId {
        next_key(self.log_entries.keys())
    }
}

impl LeadingLog {
    pub fn get(&self, id: LogEntryId) -> Option<&LeadingLogEntry> {
        self.log_entries.get(&id)
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        if self.next_committing_log_entry_id == 0 {
            None
        } else {
            Some(self.next_committing_log_entry_id - 1)
        }
    }

    pub fn begin(&mut self, r#type: LogEntryType, nodes: HashSet<NodeId>) -> LogEntryId {
        let id = self.next_log_entry_id();

        self.log_entries.insert(
            id,
            LeadingLogEntry::new(LogEntryData::new(id, r#type), nodes, HashSet::default()),
        );

        id
    }

    pub fn acknowledge(&mut self, log_entry_id: LogEntryId, node_id: NodeId) {
        self.log_entries
            .get_mut(&log_entry_id)
            .expect(&format!("Log entry with id {} should exist", log_entry_id))
            .acknowledge(node_id);

        loop {
            let log_entry = match self.log_entries.get(&self.next_committing_log_entry_id) {
                None => break,
                Some(log_entry) => log_entry,
            };

            if log_entry.is_committed() {
                self.next_committing_log_entry_id += 1;
            } else {
                break;
            }
        }
    }

    pub fn is_applicable(&self, id: LogEntryId) -> bool {
        self.log_entries
            .get(&id)
            .map(|log_entry| log_entry.is_committed())
            .unwrap_or(false)
            && self
                .last_committed_log_entry_id()
                .map(|log_entry_id| log_entry_id >= id)
                .unwrap_or(false)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use zlambda_common::log::{ClientLogEntryType, LogEntryType};

    use super::*;

    #[test]
    fn is_log_entry_uncommitted() {
        let mut entries = LeadingLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        assert_eq!(entries.get(first).unwrap().is_committed(), false);
    }

    #[test]
    fn is_log_entry_committed() {
        let mut entries = LeadingLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first, 0);
        entries.acknowledge(first, 1);

        assert_eq!(entries.get(first).unwrap().is_committed(), true);
    }

    #[test]
    #[should_panic]
    fn is_double_acknowledgement_failing() {
        let mut entries = LeadingLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first, 0);
        entries.acknowledge(first, 0);
    }

    #[test]
    #[should_panic]
    fn is_acknowledging_not_existing_log_entry_failing() {
        let mut entries = LeadingLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first + 1, 1);
    }

    #[test]
    #[should_panic]
    fn is_acknowledging_not_existing_node_failing() {
        let mut entries = LeadingLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first, 3);
    }

    #[test]
    fn are_log_entry_ids_ascending() {
        let mut entries = LeadingLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        let second = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2, 3, 4].into(),
        );

        assert_eq!(first + 1, second);
    }

    #[test]
    fn is_last_committed_id_initially_none() {
        let entries = LeadingLog::default();

        assert_eq!(entries.last_committed_log_entry_id(), None);
    }

    #[test]
    fn is_last_committed_valid_after_synchronous_acknowledgement() {
        let mut entries = LeadingLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        let second = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        let third = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first, 0);
        entries.acknowledge(first, 1);
        entries.acknowledge(first, 2);
        entries.acknowledge(second, 0);
        entries.acknowledge(second, 1);
        entries.acknowledge(second, 2);
        entries.acknowledge(third, 0);
        entries.acknowledge(third, 1);
        entries.acknowledge(third, 2);

        assert_eq!(entries.last_committed_log_entry_id(), Some(third));
    }

    #[test]
    fn is_last_committed_valid_after_asynchronous_missing_acknowledgement() {
        let mut entries = LeadingLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        let second = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        let third = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        let fourth = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first, 0);
        entries.acknowledge(first, 1);
        entries.acknowledge(first, 2);
        entries.acknowledge(second, 0);
        entries.acknowledge(second, 1);
        entries.acknowledge(second, 2);
        entries.acknowledge(third, 0);
        entries.acknowledge(fourth, 0);
        entries.acknowledge(fourth, 1);
        entries.acknowledge(fourth, 2);

        assert_eq!(entries.last_committed_log_entry_id(), Some(second));
    }

    #[test]
    fn is_last_committed_valid_after_asynchronous_recovering_acknowledgement() {
        let mut entries = LeadingLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        let second = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        let third = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        let fourth = entries.begin(
            LogEntryType::Client(ClientLogEntryType::InitializeModule),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first, 0);
        entries.acknowledge(first, 1);
        entries.acknowledge(first, 2);
        entries.acknowledge(second, 0);
        entries.acknowledge(second, 1);
        entries.acknowledge(second, 2);
        entries.acknowledge(third, 0);
        entries.acknowledge(fourth, 0);
        entries.acknowledge(fourth, 1);
        entries.acknowledge(fourth, 2);
        entries.acknowledge(third, 1);
        entries.acknowledge(third, 2);

        assert_eq!(entries.last_committed_log_entry_id(), Some(fourth));
    }
}
