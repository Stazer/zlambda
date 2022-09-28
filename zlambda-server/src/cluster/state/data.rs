use crate::algorithm::next_key;
use crate::cluster::{NodeClientId, NodeId};
use std::collections::HashMap;
use std::rc::Rc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct NodeNodesStateData {
    nodes: HashMap<NodeId, Rc<NodeNodeStateData>>,
}

impl NodeNodesStateData {
    pub fn get(&self, node_id: &NodeId) -> Option<&Rc<NodeNodeStateData>> {
        self.nodes.get(node_id)
    }

    pub fn get_mut(&mut self, node_id: &NodeId) -> Option<&mut Rc<NodeNodeStateData>> {
        self.nodes.get_mut(node_id)
    }

    pub fn insert_default(&mut self) -> NodeId {
        let id = next_key(&self.nodes);
        assert!(self
            .nodes
            .insert(id, Rc::new(NodeNodeStateData::new(id)))
            .is_none());

        id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeNodeStateData {
    id: NodeId,
}

impl NodeNodeStateData {
    pub fn new(id: NodeId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeClientTypeStateData {
    Undefined,
    Node(Rc<NodeNodeStateData>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeClientStateData {
    id: NodeClientId,
    r#type: NodeClientTypeStateData,
}

impl NodeClientStateData {
    pub fn new(id: NodeClientId, r#type: NodeClientTypeStateData) -> Self {
        Self { id, r#type }
    }

    pub fn id(&self) -> &NodeClientId {
        &self.id
    }

    pub fn r#type(&self) -> &NodeClientTypeStateData {
        &self.r#type
    }

    pub fn type_mut(&mut self) -> &mut NodeClientTypeStateData {
        &mut self.r#type
    }

    pub fn set_type(&mut self, r#type: NodeClientTypeStateData) {
        self.r#type = r#type
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct NodeClientsStateData {
    clients: HashMap<NodeClientId, NodeClientStateData>,
}

impl NodeClientsStateData {
    pub fn get(&self, node_id: &NodeId) -> Option<&NodeClientStateData> {
        self.clients.get(node_id)
    }

    pub fn get_mut(&self, node_id: &NodeId) -> Option<&NodeClientStateData> {
        self.clients.get(node_id)
    }

    pub fn insert_default(&mut self) -> NodeClientId {
        let id = next_key(&self.clients);
        assert!(self
            .clients
            .insert(id, NodeClientStateData::new(id, NodeClientTypeStateData::Undefined))
            .is_none());

        id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeStateData {
    id: NodeId,
    clients: NodeClientsStateData,
}

impl NodeStateData {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            clients: NodeClientsStateData::default(),
        }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn clients(&self) -> &NodeClientsStateData {
        &self.clients
    }

    pub fn clients_mut(&mut self) -> &NodeClientsStateData {
        &mut self.clients
    }
}
