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
    addresses: HashMap<NodeId, SocketAddr>,
    term: Term,
}

impl From<LeaderToGuestRegisterOkResponseMessage>
    for (NodeId, NodeId, HashMap<NodeId, SocketAddr>, Term)
{
    fn from(message: LeaderToGuestRegisterOkResponseMessage) -> Self {
        (
            message.node_id,
            message.leader_node_id,
            message.addresses,
            message.term,
        )
    }
}

impl LeaderToGuestRegisterOkResponseMessage {
    pub fn new(
        node_id: NodeId,
        leader_node_id: NodeId,
        addresses: HashMap<NodeId, SocketAddr>,
        term: Term,
    ) -> Self {
        Self {
            node_id,
            leader_node_id,
            addresses,
            term,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn leader_node_id(&self) -> NodeId {
        self.leader_node_id
    }

    pub fn addresses(&self) -> &HashMap<NodeId, SocketAddr> {
        &self.addresses
    }

    pub fn term(&self) -> Term {
        self.term
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaderToGuestRecoveryErrorResponseMessage {
    message: String,
}

impl From<LeaderToGuestRecoveryErrorResponseMessage> for (String,) {
    fn from(message: LeaderToGuestRecoveryErrorResponseMessage) -> Self {
        (message.message,)
    }
}

impl LeaderToGuestRecoveryErrorResponseMessage {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &String {
        &self.message
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaderToGuestRecoveryOkResponseMessage {
    leader_node_id: NodeId,
}

impl From<LeaderToGuestRecoveryOkResponseMessage> for (NodeId,) {
    fn from(message: LeaderToGuestRecoveryOkResponseMessage) -> Self {
        (message.leader_node_id,)
    }
}

impl LeaderToGuestRecoveryOkResponseMessage {
    pub fn new(leader_node_id: NodeId) -> Self {
        Self { leader_node_id }
    }

    pub fn leader_node_id(&self) -> NodeId {
        self.leader_node_id
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
