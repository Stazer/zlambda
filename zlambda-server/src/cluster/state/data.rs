use crate::cluster::{ClientId, ConnectionId, NodeId};
use std::collections::HashMap;
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ClientData {
    pub(super) id: ClientId,
    pub(super) connection_id: ConnectionId,
}

impl ClientData {
    pub fn new(id: ClientId, connection_id: ConnectionId) -> Self {
        Self { id, connection_id }
    }

    pub fn id(&self) -> ClientId {
        self.id
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct NodeData {
    pub(super) id: NodeId,
    pub(super) connection_id: Option<ConnectionId>,
    pub(super) address: SocketAddr,
}

impl NodeData {
    pub fn new(id: NodeId, connection_id: Option<ConnectionId>, address: SocketAddr) -> Self {
        Self {
            id,
            connection_id,
            address,
        }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn connection_id(&self) -> Option<ConnectionId> {
        self.connection_id
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub enum ConnectionTypeData {
    Node {
        node_id: NodeId,

    },
    Client {
        client_id: ClientId,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct ConnectionData {
    pub(super) id: ConnectionId,
    pub(super) r#type: Option<ConnectionTypeData>,
}

impl ConnectionData {
    pub fn new(id: ConnectionId, r#type: Option<ConnectionTypeData>) -> Self {
        Self { id, r#type }
    }

    pub fn id(&self) -> &ConnectionId {
        &self.id
    }

    pub fn r#type(&self) -> &Option<ConnectionTypeData> {
        &self.r#type
    }

    pub fn type_mut(&mut self) -> &mut Option<ConnectionTypeData> {
        &mut self.r#type
    }

    pub fn set_type(&mut self, r#type: Option<ConnectionTypeData>) {
        self.r#type = r#type
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default)]
pub struct StateData {
    pub(super) node_id: Option<NodeId>,
    pub(super) connections: HashMap<ConnectionId, ConnectionData>,
    pub(super) nodes: HashMap<NodeId, NodeData>,
    pub(super) clients: HashMap<ClientId, ClientData>,
}

impl StateData {
    pub fn node_id(&self) -> Option<NodeId> {
        self.node_id
    }

    pub fn connections(&self) -> &HashMap<ConnectionId, ConnectionData> {
        &self.connections
    }

    pub fn nodes(&self) -> &HashMap<NodeId, NodeData> {
        &self.nodes
    }

    pub fn clients(&self) -> &HashMap<ClientId, ClientData> {
        &self.clients
    }
}
