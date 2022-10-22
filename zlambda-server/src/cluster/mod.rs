mod node;
mod packet;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use node::*;
pub use packet::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::algorithm::next_key;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClientId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NodeId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ConnectionId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type TermId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LogEntryId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LogEntryType {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogEntry {
    id: LogEntryId,
    term_id: TermId,
    r#type: LogEntryType,
}

impl LogEntry {
    pub fn new(id: LogEntryId, term_id: TermId, r#type: LogEntryType) -> Self {
        Self {
            id,
            term_id,
            r#type,
        }
    }

    pub fn id(&self) -> LogEntryId {
        self.id
    }

    pub fn term_id(&self) -> TermId {
        self.term_id
    }

    pub fn r#type(&self) -> &LogEntryType {
        &self.r#type
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum LeaderLogEntryState {
    Uncommitted,
    Committed,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderLogEntry {
    data: LogEntry,
    state: LeaderLogEntryState,
    acknowledged_begin_node_ids: HashSet<NodeId>,
}

impl LeaderLogEntry {
    pub fn data(&self) -> &LogEntry {
        &self.data
    }

    pub fn state(&self) -> &LeaderLogEntryState {
        &self.state
    }

    pub fn acknowledged_begin_node_ids(&self) -> &HashSet<NodeId> {
        &self.acknowledged_begin_node_ids
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct LeaderLog {
    entries: HashMap<LogEntryId, LeaderLogEntry>,
}

impl LeaderLog {
    pub fn entries(&self) -> &HashMap<LogEntryId, LeaderLogEntry> {
        &self.entries
    }

    pub fn begin(&mut self, term_id: TermId, r#type: LogEntryType) -> LogEntryId {
        let id = next_key(&self.entries);

        self.entries.insert(
            id,
            LeaderLogEntry {
                data: LogEntry {
                    id,
                    term_id,
                    r#type,
                },
                state: LeaderLogEntryState::Uncommitted,
                acknowledged_begin_node_ids: HashSet::default(),
            },
        );

        0
    }

    pub fn acknowledge_begin(&mut self, id: LogEntryId, node_id: NodeId) {
        match self.entries.get_mut(&id) {
            Some(entry) => {
                entry.acknowledged_begin_node_ids.insert(node_id);
            }
            None => {}
        };
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum FollowerLogEntryState {
    Uncommitted,
    Committed,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerLogEntry {
    data: LogEntry,
    state: FollowerLogEntryState,
}

impl FollowerLogEntry {
    pub fn data(&self) -> &LogEntry {
        &self.data
    }

    pub fn state(&self) -> &FollowerLogEntryState {
        &self.state
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct FollowerLog {
    entries: HashMap<LogEntryId, FollowerLogEntry>,
}

impl FollowerLog {
    pub fn entries(&self) -> &HashMap<LogEntryId, FollowerLogEntry> {
        &self.entries
    }

    pub fn begin(&mut self, data: LogEntry) {
        self.entries.insert(
            data.id(),
            FollowerLogEntry {
                data,
                state: FollowerLogEntryState::Uncommitted,
            },
        );
    }

    pub fn commit(&mut self, id: LogEntryId) {
        match self.entries.get_mut(&id) {
            Some(entry) => entry.state = FollowerLogEntryState::Committed,
            None => {}
        };
    }
}
