use crate::algorithm::next_key;
use crate::cluster::{
    ClientId, ConnectionId, LogEntry, LogEntryId, LogEntryType, NodeActor, NodeId, TermId,
    LeaderNodeFollowerActor,
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
