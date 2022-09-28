use crate::algorithm::next_key;
use crate::cluster::{NodeClientId, NodeId};
use std::collections::HashMap;
use std::rc::Rc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct NodeNodesState {
    nodes: HashMap<NodeId, Rc<NodeNodeState>>,
}

impl NodeNodesState {
    pub fn get(&self, node_id: &NodeId) -> Option<&Rc<NodeNodeState>> {
        self.nodes.get(node_id)
    }

    pub fn get_mut(&mut self, node_id: &NodeId) -> Option<&mut Rc<NodeNodeState>> {
        self.nodes.get_mut(node_id)
    }

    pub fn insert_default(&mut self) -> NodeId {
        let id = next_key(&self.nodes);
        assert!(self
            .nodes
            .insert(id, Rc::new(NodeNodeState::new(id)))
            .is_none());

        id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeNodeState {
    id: NodeId,
}

impl NodeNodeState {
    pub fn new(id: NodeId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeClientTypeState {
    Undefined,
    Node(Rc<NodeNodeState>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeClientState {
    id: NodeClientId,
    r#type: NodeClientTypeState,
}

impl NodeClientState {
    pub fn new(id: NodeClientId, r#type: NodeClientTypeState) -> Self {
        Self { id, r#type }
    }

    pub fn id(&self) -> &NodeClientId {
        &self.id
    }

    pub fn r#type(&self) -> &NodeClientTypeState {
        &self.r#type
    }

    pub fn type_mut(&mut self) -> &mut NodeClientTypeState {
        &mut self.r#type
    }

    pub fn set_type(&mut self, r#type: NodeClientTypeState) {
        self.r#type = r#type
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct NodeClientsState {
    clients: HashMap<NodeClientId, NodeClientState>,
}

impl NodeClientsState {
    pub fn get(&self, node_id: &NodeId) -> Option<&NodeClientState> {
        self.clients.get(node_id)
    }

    pub fn get_mut(&self, node_id: &NodeId) -> Option<&NodeClientState> {
        self.clients.get(node_id)
    }

    pub fn insert_default(&mut self) -> NodeClientId {
        let id = next_key(&self.clients);
        assert!(self
            .clients
            .insert(id, NodeClientState::new(id, NodeClientTypeState::Undefined))
            .is_none());

        id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeState {
    id: NodeId,
    clients: NodeClientsState,
}

impl NodeState {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            clients: NodeClientsState::default(),
        }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn clients(&self) -> &NodeClientsState {
        &self.clients
    }

    pub fn clients_mut(&mut self) -> &NodeClientsState {
        &mut self.clients
    }
}
