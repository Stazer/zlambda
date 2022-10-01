use crate::algorithm::next_key;
use crate::cluster::{NodeConnectionId, NodeId};
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
pub enum NodeConnectionTypeStateData {
    Undefined,
    Node(Rc<NodeNodeStateData>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeConnectionStateData {
    id: NodeConnectionId,
    r#type: NodeConnectionTypeStateData,
}

impl NodeConnectionStateData {
    pub fn new(id: NodeConnectionId, r#type: NodeConnectionTypeStateData) -> Self {
        Self { id, r#type }
    }

    pub fn id(&self) -> &NodeConnectionId {
        &self.id
    }

    pub fn r#type(&self) -> &NodeConnectionTypeStateData {
        &self.r#type
    }

    pub fn type_mut(&mut self) -> &mut NodeConnectionTypeStateData {
        &mut self.r#type
    }

    pub fn set_type(&mut self, r#type: NodeConnectionTypeStateData) {
        self.r#type = r#type
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct NodeConnectionsStateData {
    connections: HashMap<NodeConnectionId, NodeConnectionStateData>,
}

impl NodeConnectionsStateData {
    pub fn get(&self, node_id: &NodeId) -> Option<&NodeConnectionStateData> {
        self.connections.get(node_id)
    }

    pub fn get_mut(&self, node_id: &NodeId) -> Option<&NodeConnectionStateData> {
        self.connections.get(node_id)
    }

    pub fn insert_default(&mut self) -> NodeConnectionId {
        let id = next_key(&self.connections);
        assert!(self
            .connections
            .insert(
                id,
                NodeConnectionStateData::new(id, NodeConnectionTypeStateData::Undefined)
            )
            .is_none());

        id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeStateData {
    id: NodeId,
    connections: NodeConnectionsStateData,
    nodes: NodeNodesStateData,
}

impl NodeStateData {
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            connections: NodeConnectionsStateData::default(),
            nodes: NodeNodesStateData::default(),
        }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn connections(&self) -> &NodeConnectionsStateData {
        &self.connections
    }

    pub fn connections_mut(&mut self) -> &NodeConnectionsStateData {
        &mut self.connections
    }

    pub fn nodes(&self) -> &NodeNodesStateData {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut NodeNodesStateData {
        &mut self.nodes
    }
}
