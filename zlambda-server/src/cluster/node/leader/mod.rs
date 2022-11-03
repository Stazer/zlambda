mod actor;
mod client;
mod connection;
mod follower;
mod message;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use actor::*;
pub use client::*;
pub use connection::*;
pub use follower::*;
pub use message::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::algorithm::next_key;
use crate::cluster::{
    ClientId, ConnectionId, LogEntry, LogEntryId, LogEntryType, NodeActor, NodeId, TermId,
};
use crate::common::{TcpListenerActor, TcpListenerActorAcceptMessage, UpdateRecipientActorMessage};
use actix::dev::{MessageResponse, OneshotSender};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::collections::hash_map::RandomState;
use std::collections::hash_set::Difference;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Debug, Formatter};
use std::iter::once;
use std::net::SocketAddr;
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type CommittedLogEntry = LogEntry;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct UncommittedLogEntry {
    id: LogEntryId,
    r#type: LogEntryType,
    acknowledging_nodes: HashSet<NodeId>,
    acknowledged_nodes: HashSet<NodeId>,
}

impl From<UncommittedLogEntry> for CommittedLogEntry {
    fn from(entry: UncommittedLogEntry) -> Self {
        Self::new(entry.id, entry.r#type)
    }
}

impl UncommittedLogEntry {
    pub fn new(
        id: LogEntryId,
        r#type: LogEntryType,
        acknowledging_nodes: HashSet<NodeId>,
        acknowledged_nodes: HashSet<NodeId>,
    ) -> Self {
        Self {
            id,
            r#type,
            acknowledging_nodes,
            acknowledged_nodes,
        }
    }

    pub fn id(&self) -> LogEntryId {
        self.id
    }

    pub fn r#type(&self) -> &LogEntryType {
        &self.r#type
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
                "log entry {} does not need to be acknowledged by node {}",
                self.id, node_id,
            );
        }

        if self.acknowledged_nodes.contains(&node_id) {
            panic!(
                "Log entry {} already acknowledged by node {}",
                self.id, node_id
            );
        }

        self.acknowledging_nodes.remove(&node_id);
        self.acknowledged_nodes.insert(node_id);
    }

    pub fn committable(&self) -> bool {
        self.remaining_acknowledging_nodes().count() <= self.quorum_count()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct LeaderNodeActorLogEntries {
    uncommitted: HashMap<LogEntryId, UncommittedLogEntry>,
    committed: HashMap<LogEntryId, CommittedLogEntry>,
    last_committed_id: LogEntryId,
}

impl LeaderNodeActorLogEntries {
    pub fn next_key(&self) -> LogEntryId {
        next_key(self.uncommitted.keys().chain(self.committed.keys()))
    }

    pub fn begin(&mut self, r#type: LogEntryType, nodes: HashSet<NodeId>) -> LogEntryId {
        let id = self.next_key();

        self.uncommitted.insert(
            id,
            UncommittedLogEntry::new(id, r#type, nodes, HashSet::default()),
        );

        id
    }

    pub fn is_uncommitted(&self, log_entry_id: LogEntryId) -> bool {
        self.uncommitted.get(&log_entry_id).is_some()
    }

    pub fn acknowledge(&mut self, log_entry_id: LogEntryId, node_id: NodeId) -> Option<LogEntryId> {
        let log_entry = self
            .uncommitted
            .get_mut(&log_entry_id)
            .expect("Log entry should be existing as uncommitted");

        log_entry.acknowledge(node_id);

        if log_entry.committable() {
            let _ = log_entry;

            self.committed.insert(
                log_entry_id,
                self.uncommitted
                    .remove(&log_entry_id)
                    .expect("Log entry should exist")
                    .into(),
            );

            self.last_committed_id = next_key(self.committed.keys());

            Some(log_entry_id)
        } else {
            None
        }
    }

    pub fn last_committed_id(&self) -> LogEntryId {
        self.last_committed_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderNodeActorActorAddresses {
    node: Addr<NodeActor>,
    tcp_listener: Addr<TcpListenerActor>,
    pub follower: HashMap<NodeId, Addr<LeaderNodeFollowerActor>>,
    clients: HashMap<ClientId, Addr<LeaderNodeClientActor>>,
}

impl Debug for LeaderNodeActorActorAddresses {
    fn fmt(&self, _formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl LeaderNodeActorActorAddresses {
    pub fn new(
        node: Addr<NodeActor>,
        tcp_listener: Addr<TcpListenerActor>,
        follower: HashMap<NodeId, Addr<LeaderNodeFollowerActor>>,
        clients: HashMap<ClientId, Addr<LeaderNodeClientActor>>,
    ) -> Self {
        Self {
            node,
            tcp_listener,
            follower,
            clients,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acknowledgement() {

    }
}
