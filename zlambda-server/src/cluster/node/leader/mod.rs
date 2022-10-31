mod client;
mod connection;
mod follower;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use client::*;
pub use connection::*;
pub use follower::*;

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

#[derive(Debug)]
pub struct DestroyConnectionActorMessage {
    connection_id: ConnectionId,
}

impl From<DestroyConnectionActorMessage> for (ConnectionId,) {
    fn from(message: DestroyConnectionActorMessage) -> Self {
        (message.connection_id,)
    }
}

impl Message for DestroyConnectionActorMessage {
    type Result = ();
}

impl DestroyConnectionActorMessage {
    pub fn new(connection_id: ConnectionId) -> Self {
        Self { connection_id }
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CreateFollowerActorMessageResponse {
    node_id: NodeId,
    leader_node_id: NodeId,
    term_id: TermId,
    node_socket_addresses: HashMap<NodeId, SocketAddr>,
}

impl From<CreateFollowerActorMessageResponse>
    for (NodeId, NodeId, TermId, HashMap<NodeId, SocketAddr>)
{
    fn from(result: CreateFollowerActorMessageResponse) -> Self {
        (
            result.node_id,
            result.leader_node_id,
            result.term_id,
            result.node_socket_addresses,
        )
    }
}

impl<A, M> MessageResponse<A, M> for CreateFollowerActorMessageResponse
where
    A: Actor,
    M: Message<Result = Self>,
{
    fn handle(self, _context: &mut A::Context, sender: Option<OneshotSender<M::Result>>) {
        if let Some(sender) = sender {
            sender
                .send(self)
                .expect("Cannot send FollowerUpgradeActorMessageResponse");
        }
    }
}

impl CreateFollowerActorMessageResponse {
    pub fn new(
        node_id: NodeId,
        leader_node_id: NodeId,
        term_id: TermId,
        node_socket_addresses: HashMap<NodeId, SocketAddr>,
    ) -> Self {
        Self {
            node_id,
            leader_node_id,
            term_id,
            node_socket_addresses,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn leader_node_id(&self) -> NodeId {
        self.leader_node_id
    }

    pub fn term_id(&self) -> TermId {
        self.term_id
    }

    pub fn node_socket_addresses(&self) -> &HashMap<NodeId, SocketAddr> {
        &self.node_socket_addresses
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CreateFollowerActorMessage {
    socket_address: SocketAddr,
    follower_actor_address: Addr<LeaderNodeFollowerActor>,
}

impl From<CreateFollowerActorMessage> for (SocketAddr, Addr<LeaderNodeFollowerActor>) {
    fn from(message: CreateFollowerActorMessage) -> Self {
        (message.socket_address, message.follower_actor_address)
    }
}

impl Message for CreateFollowerActorMessage {
    type Result = CreateFollowerActorMessageResponse;
}

impl CreateFollowerActorMessage {
    pub fn new(
        socket_address: SocketAddr,
        follower_actor_address: Addr<LeaderNodeFollowerActor>,
    ) -> Self {
        Self {
            socket_address,
            follower_actor_address,
        }
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }

    pub fn follower_actor_address(&self) -> &Addr<LeaderNodeFollowerActor> {
        &self.follower_actor_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DestroyFollowerActorMessage {
    node_id: NodeId,
}

impl From<DestroyFollowerActorMessage> for (NodeId,) {
    fn from(message: DestroyFollowerActorMessage) -> Self {
        (message.node_id,)
    }
}

impl Message for DestroyFollowerActorMessage {
    type Result = ();
}

impl DestroyFollowerActorMessage {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct AcknowledgeLogEntryActorMessage {
    log_entry_id: LogEntryId,
    node_id: NodeId,
}

impl From<AcknowledgeLogEntryActorMessage> for (LogEntryId, NodeId) {
    fn from(message: AcknowledgeLogEntryActorMessage) -> Self {
        (message.log_entry_id, message.node_id)
    }
}

impl Message for AcknowledgeLogEntryActorMessage {
    type Result = ();
}

impl AcknowledgeLogEntryActorMessage {
    pub fn new(log_entry_id: LogEntryId, node_id: NodeId) -> Self {
        Self {
            log_entry_id,
            node_id,
        }
    }

    pub fn log_entry_id(&self) -> LogEntryId {
        self.log_entry_id
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct BeginLogEntryActorMessage {
    log_entry_type: LogEntryType,
}

impl From<BeginLogEntryActorMessage> for (LogEntryType,) {
    fn from(message: BeginLogEntryActorMessage) -> Self {
        (message.log_entry_type,)
    }
}

impl Message for BeginLogEntryActorMessage {
    type Result = ();
}

impl BeginLogEntryActorMessage {
    pub fn new(log_entry_type: LogEntryType) -> Self {
        Self { log_entry_type }
    }

    pub fn log_entry_type(&self) -> &LogEntryType {
        &self.log_entry_type
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderNodeActorActorAddresses {
    node: Addr<NodeActor>,
    tcp_listener: Addr<TcpListenerActor>,
    follower: HashMap<NodeId, Addr<LeaderNodeFollowerActor>>,
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

            Some(log_entry_id)
        } else {
            None
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeActor {
    actor_addresses: LeaderNodeActorActorAddresses,

    term_id: TermId,
    node_id: NodeId,
    node_socket_addresses: HashMap<NodeId, SocketAddr>,

    log_entries: LeaderNodeActorLogEntries,
}

impl Actor for LeaderNodeActor {
    type Context = Context<Self>;
}

impl Handler<CreateFollowerActorMessage> for LeaderNodeActor {
    type Result = <CreateFollowerActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: CreateFollowerActorMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let node_id = next_key(
            self.actor_addresses
                .follower
                .keys()
                .chain(once(&self.node_id)),
        );

        let (socket_address, follower_actor_address) = message.into();
        self.actor_addresses
            .follower
            .insert(node_id, follower_actor_address);
        self.node_socket_addresses.insert(node_id, socket_address);

        context.notify(BeginLogEntryActorMessage::new(LogEntryType::UpdateNodes(
            self.node_socket_addresses.clone(),
        )));

        Self::Result::new(
            node_id,
            self.node_id,
            self.term_id,
            self.node_socket_addresses.clone(),
        )
    }
}

impl Handler<DestroyFollowerActorMessage> for LeaderNodeActor {
    type Result = <DestroyFollowerActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: DestroyFollowerActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (node_id,) = message.into();
        self.actor_addresses.follower.remove(&node_id);
    }
}

impl Handler<TcpListenerActorAcceptMessage> for LeaderNodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result,) = message.into();

        match result {
            Ok(tcp_stream) => {
                LeaderNodeConnectionActor::new(context.address(), tcp_stream);
            }
            Err(error) => {
                error!("{}", error);
                context.stop();
            }
        }
    }
}

impl Handler<BeginLogEntryActorMessage> for LeaderNodeActor {
    type Result = <BeginLogEntryActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: BeginLogEntryActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (r#type,) = message.into();

        let log_entry_id = self.log_entries.begin(
            r#type.clone(),
            self.node_socket_addresses.keys().copied().collect(),
        );

        for node_id in self
            .log_entries
            .uncommitted
            .get(&log_entry_id)
            .expect("Log entry should be uncommitted")
            .remaining_acknowledging_nodes()
            .filter(|x| **x != self.node_id)
        {
            self.actor_addresses
                .follower
                .get(node_id)
                .expect("Node id should have an follower actor address")
                .try_send(ReplicateLogEntryActorMessage::new(LogEntry::new(
                    log_entry_id,
                    r#type.clone(),
                )))
                .expect("Sending ReplicateLogEntryActorMessage should be successful");
        }

        match self.log_entries.acknowledge(log_entry_id, self.node_id) {
            Some(log_entry_id) => {
                trace!("Replicated {} on one node", log_entry_id);
            }
            None => {}
        }
    }
}

impl Handler<AcknowledgeLogEntryActorMessage> for LeaderNodeActor {
    type Result = <AcknowledgeLogEntryActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: AcknowledgeLogEntryActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        if self.log_entries.is_uncommitted(message.log_entry_id()) {
            match self
                .log_entries
                .acknowledge(message.log_entry_id(), message.node_id())
            {
                Some(log_entry_id) => {
                    trace!("Replicated {} in cluster", log_entry_id);
                }
                None => {}
            }
        }

        trace!(
            "Acknowledged {} on node {}",
            message.log_entry_id(),
            message.node_id(),
        );
    }
}

impl LeaderNodeActor {
    pub fn new(
        node_actor_address: Addr<NodeActor>,
        tcp_listener_actor_address: Addr<TcpListenerActor>,
        term_id: TermId,
        node_id: NodeId,
        node_socket_addresses: HashMap<NodeId, SocketAddr>,
    ) -> Addr<LeaderNodeActor> {
        Self::create(move |context| {
            tcp_listener_actor_address.do_send(UpdateRecipientActorMessage::new(
                context.address().recipient(),
            ));

            Self {
                actor_addresses: LeaderNodeActorActorAddresses::new(
                    node_actor_address,
                    tcp_listener_actor_address,
                    HashMap::default(),
                    HashMap::default(),
                ),

                term_id,
                node_id,
                node_socket_addresses,

                log_entries: LeaderNodeActorLogEntries::default(),
            }
        })
    }
}
