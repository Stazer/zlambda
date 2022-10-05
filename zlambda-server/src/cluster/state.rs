use crate::algorithm::next_key;
use crate::cluster::{ConnectionId, NodeId};
use std::collections::{HashMap, HashSet};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ConnectionStateType {
    RegisteringNode,
    RegisteredNode,
    Client,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConnectionState {
    id: ConnectionId,
    r#type: Option<ConnectionStateType>,
}

impl ConnectionState {
    pub fn new(id: ConnectionId, r#type: Option<ConnectionStateType>) -> Self {
        Self { id, r#type }
    }

    pub fn id(&self) -> ConnectionId {
        self.id
    }

    pub fn r#type(&self) -> &Option<ConnectionStateType> {
        &self.r#type
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct NodeState {
    id: NodeId,
    connections: HashMap<ConnectionId, ConnectionState>,
    //nodes: HashMap<NodeId, NodeState>,
}

impl NodeState {
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn connections(&self) -> &HashMap<ConnectionId, ConnectionState> {
        &self.connections
    }

    pub fn create_connection(&mut self) -> ConnectionId {
        let connection_id = next_key(&self.connections);
        assert!(self
            .connections
            .insert(connection_id, ConnectionState::new(connection_id, None,))
            .is_none());

        connection_id
    }

    pub fn destroy_connection(&mut self, connection_id: ConnectionId) {
        assert!(self.connections.remove(&connection_id).is_some());
    }

    pub fn handle_connect(&mut self) -> ConnectionId {
        let connection_id = next_key(&self.connections);
        assert!(self
            .connections
            .insert(connection_id, ConnectionState::new(connection_id, None,))
            .is_none());

        connection_id
    }

    pub fn handle_disconnect(&mut self, connection_id: ConnectionId) {
        assert!(self.connections.remove(&connection_id).is_some());
    }

    pub fn handle_node_handshake_request(&mut self, node_id: NodeId) {}

    pub fn handle_node_register_request(&mut self, connection_id: ConnectionId) {
        let connection = self
            .connections
            .get_mut(&connection_id)
            .expect("Connection not found");
        assert!(connection.r#type.is_none());
        connection.r#type = Some(ConnectionStateType::RegisteringNode);
    }
}
