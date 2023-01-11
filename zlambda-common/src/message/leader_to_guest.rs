use crate::error::RecoveryError;
use crate::message::{BasicMessageStreamReader, BasicMessageStreamWriter, Message, MessageError};
use crate::node::NodeId;
use crate::term::Term;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaderToGuestRegisterOkResponseMessage {
    node_id: NodeId,
    leader_node_id: NodeId,
    node_addresses: Vec<SocketAddr>,
    term: Term,
}

impl From<LeaderToGuestRegisterOkResponseMessage> for (NodeId, NodeId, Vec<SocketAddr>, Term) {
    fn from(message: LeaderToGuestRegisterOkResponseMessage) -> Self {
        (
            message.node_id,
            message.leader_node_id,
            message.node_addresses,
            message.term,
        )
    }
}

impl LeaderToGuestRegisterOkResponseMessage {
    pub fn new(
        node_id: NodeId,
        leader_node_id: NodeId,
        node_addresses: Vec<SocketAddr>,
        term: Term,
    ) -> Self {
        Self {
            node_id,
            leader_node_id,
            node_addresses,
            term,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn leader_node_id(&self) -> NodeId {
        self.leader_node_id
    }

    pub fn node_addresses(&self) -> &Vec<SocketAddr> {
        &self.node_addresses
    }

    pub fn term(&self) -> Term {
        self.term
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaderToGuestRecoveryErrorResponseMessage {
    error: RecoveryError,
}

impl From<LeaderToGuestRecoveryErrorResponseMessage> for (RecoveryError,) {
    fn from(message: LeaderToGuestRecoveryErrorResponseMessage) -> Self {
        (message.error,)
    }
}

impl LeaderToGuestRecoveryErrorResponseMessage {
    pub fn new(error: RecoveryError) -> Self {
        Self { error }
    }

    pub fn error(&self) -> &RecoveryError {
        &self.error
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaderToGuestRecoveryOkResponseMessage {
    node_id: NodeId,
    leader_node_id: NodeId,
    node_addresses: Vec<SocketAddr>,
    term: Term,
}

impl From<LeaderToGuestRecoveryOkResponseMessage> for (NodeId, NodeId, Vec<SocketAddr>, Term) {
    fn from(message: LeaderToGuestRecoveryOkResponseMessage) -> Self {
        (
            message.node_id,
            message.leader_node_id,
            message.node_addresses,
            message.term,
        )
    }
}

impl LeaderToGuestRecoveryOkResponseMessage {
    pub fn new(
        node_id: NodeId,
        leader_node_id: NodeId,
        node_addresses: Vec<SocketAddr>,
        term: Term,
    ) -> Self {
        Self {
            node_id,
            leader_node_id,
            node_addresses,
            term,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn leader_node_id(&self) -> NodeId {
        self.leader_node_id
    }

    pub fn node_addresses(&self) -> &Vec<SocketAddr> {
        &self.node_addresses
    }

    pub fn term(&self) -> Term {
        self.term
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum LeaderToGuestMessage {
    RegisterOkResponse(LeaderToGuestRegisterOkResponseMessage),
    RecoveryErrorResponse(LeaderToGuestRecoveryErrorResponseMessage),
    RecoveryOkResponse(LeaderToGuestRecoveryOkResponseMessage),
}

impl From<Message> for Result<LeaderToGuestMessage, MessageError> {
    fn from(message: Message) -> Self {
        match message {
            Message::LeaderToGuest(message) => Ok(message),
            _ => Err(MessageError::UnexpectedMessage(message)),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type LeaderToGuestMessageStreamReader = BasicMessageStreamReader<LeaderToGuestMessage>;
pub type LeaderToGuestMessageStreamWriter = BasicMessageStreamWriter<LeaderToGuestMessage>;
