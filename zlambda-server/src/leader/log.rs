use std::collections::hash_map::RandomState;
use std::collections::hash_set::Difference;
use std::collections::HashSet;
use zlambda_common::log::{LogEntryData, LogEntryId, LogEntryType};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderLogEntry {
    data: LogEntryData,
    acknowledging_nodes: HashSet<NodeId>,
    acknowledged_nodes: HashSet<NodeId>,
}

impl LeaderLogEntry {
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
pub struct LeaderLog {
    log_entries: Vec<LeaderLogEntry>,
    next_committing_log_entry_id: LogEntryId,
}

impl LeaderLog {
    fn next_log_entry_id(&self) -> LogEntryId {
        self.log_entries.len()
    }
}

impl LeaderLog {
    pub fn get(&self, id: LogEntryId) -> Option<&LeaderLogEntry> {
        self.log_entries.get(id)
    }

    pub fn last_committed_log_entry_id(&self) -> Option<LogEntryId> {
        if self.next_committing_log_entry_id == 0 {
            None
        } else {
            Some(self.next_committing_log_entry_id - 1)
        }
    }

    pub fn begin(
        &mut self,
        r#type: LogEntryType,
        term: Term,
        nodes: HashSet<NodeId>,
    ) -> LogEntryId {
        let id = self.next_log_entry_id();

        self.log_entries.push(LeaderLogEntry::new(
            LogEntryData::new(id, r#type, term),
            nodes,
            HashSet::default(),
        ));

        id
    }

    pub fn acknowledge(&mut self, log_entry_id: LogEntryId, node_id: NodeId) -> Vec<LogEntryId> {
        let log_entry = self
            .log_entries
            .get_mut(log_entry_id)
            .expect(&format!("Log entry with id {} should exist", log_entry_id));

        log_entry.acknowledge(node_id);

        let mut committed_log_entry_ids = Vec::default();

        loop {
            let log_entry = match self.log_entries.get(self.next_committing_log_entry_id) {
                None => break,
                Some(log_entry) => log_entry,
            };

            if log_entry.is_committed() {
                committed_log_entry_ids.push(self.next_committing_log_entry_id);

                self.next_committing_log_entry_id += 1;
            } else {
                break;
            }
        }

        committed_log_entry_ids
    }

    pub fn is_applicable(&self, id: LogEntryId) -> bool {
        self.log_entries
            .get(id)
            .map(|log_entry| log_entry.is_committed())
            .unwrap_or(false)
            && self
                .last_committed_log_entry_id()
                .map(|log_entry_id| log_entry_id >= id)
                .unwrap_or(false)
    }

    pub fn acknowledging_log_entries(&self, node_id: NodeId) -> Vec<LogEntryData> {
        let mut result = Vec::default();

        for log_entry in self.log_entries.iter() {
            if log_entry.acknowledging_nodes().contains(&node_id)
                && !log_entry.acknowledged_nodes().contains(&node_id)
            {
                result.push(log_entry.data().clone());
            }
        }

        result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use zlambda_common::log::{ClientLogEntryType, LogEntryType};

    use super::*;

    #[test]
    fn is_log_entry_uncommitted() {
        let mut entries = LeaderLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        assert_eq!(entries.get(first).unwrap().is_committed(), false);
    }

    #[test]
    fn is_log_entry_committed() {
        let mut entries = LeaderLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first, 0);
        entries.acknowledge(first, 1);

        assert_eq!(entries.get(first).unwrap().is_committed(), true);
    }

    #[test]
    #[should_panic]
    fn is_double_acknowledgement_failing() {
        let mut entries = LeaderLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first, 0);
        entries.acknowledge(first, 0);
    }

    #[test]
    #[should_panic]
    fn is_acknowledging_not_existing_log_entry_failing() {
        let mut entries = LeaderLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first + 1, 1);
    }

    #[test]
    #[should_panic]
    fn is_acknowledging_not_existing_node_failing() {
        let mut entries = LeaderLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        entries.acknowledge(first, 3);
    }

    #[test]
    fn are_log_entry_ids_ascending() {
        let mut entries = LeaderLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        let second = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2, 3, 4].into(),
        );

        assert_eq!(first + 1, second);
    }

    #[test]
    fn is_last_committed_id_initially_none() {
        let entries = LeaderLog::default();

        assert_eq!(entries.last_committed_log_entry_id(), None);
    }

    #[test]
    fn is_last_committed_valid_after_synchronous_acknowledgement() {
        let mut entries = LeaderLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        let second = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        let third = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
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
        let mut entries = LeaderLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        let second = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        let third = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        let fourth = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
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
        let mut entries = LeaderLog::default();
        let first = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        let second = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        let third = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
            [0, 1, 2].into(),
        );
        let fourth = entries.begin(
            LogEntryType::Client(ClientLogEntryType::Initialize),
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
