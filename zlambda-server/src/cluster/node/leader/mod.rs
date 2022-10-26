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

    pub fn quorum_count(&self) -> usize {
        self.acknowledging_nodes.len() / 2
    }

    pub fn acknowledge(&mut self, node_id: NodeId) {
        if !self.acknowledging_nodes.contains(&node_id) {
            panic!(
                "Cannot acknowledge log entry {} by node {}",
                self.id, node_id
            );
        }

        if self.acknowledged_nodes.contains(&node_id) {
            panic!(
                "Log entry {} already acknowledged by node {}",
                self.id, node_id
            );
        }
    }

    pub fn committable(&self) -> bool {
        self.acknowledging_nodes
            .intersection(&self.acknowledged_nodes)
            .count()
            > self.quorum_count()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct LeaderNodeActorLogEntries {
    uncommitted: HashMap<LogEntryId, UncommittedLogEntry>,
    committed: HashMap<LogEntryId, CommittedLogEntry>,
}

impl LeaderNodeActorLogEntries {
    pub fn acknowledge(&mut self, log_entry_id: LogEntryId, node_id: NodeId) -> Option<LogEntryId> {
        let log_entry = self
            .uncommitted
            .get_mut(&log_entry_id)
            .expect("Log entry should exist");
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
        _context: &mut <Self as Actor>::Context,
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

impl Handler<AcknowledgeLogEntryActorMessage> for LeaderNodeActor {
    type Result = <AcknowledgeLogEntryActorMessage as Message>::Result;

    fn handle(
        &mut self,
        message: AcknowledgeLogEntryActorMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        self.log_entries
            .acknowledge(message.log_entry_id(), message.node_id());
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
