use crate::node::member::NodeMemberReference;
use crate::node::NodeId;
use crate::term::Term;
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeFollowerRegistrationAttemptOutput {
    node_id: NodeId,
    leader_node_id: NodeId,
    node_socket_addresses: Vec<SocketAddr>,
    term: Term,
}

impl From<NodeFollowerRegistrationAttemptOutput> for (NodeId, NodeId, Vec<SocketAddr>, Term) {
    fn from(output: NodeFollowerRegistrationAttemptOutput) -> Self {
        (
            output.node_id,
            output.leader_node_id,
            output.node_socket_addresses,
            output.term,
        )
    }
}

impl NodeFollowerRegistrationAttemptOutput {
    pub fn new(
        node_id: NodeId,
        leader_node_id: NodeId,
        node_socket_addresses: Vec<SocketAddr>,
        term: Term,
    ) -> Self {
        Self {
            node_id,
            leader_node_id,
            node_socket_addresses,
            term,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn leader_node_id(&self) -> NodeId {
        self.leader_node_id
    }

    pub fn node_socket_addresses(&self) -> &Vec<SocketAddr> {
        &self.node_socket_addresses
    }

    pub fn term(&self) -> Term {
        self.term
    }
}
