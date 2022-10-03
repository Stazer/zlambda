use crate::algorithm::next_key;
use crate::cluster::{NodeConnectionId, NodeId};
use std::collections::HashMap;
use std::rc::Rc;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ConnectionTypeNodeStateData {
    Node,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ConnectionNodeStateData {
    pub(super) r#type: Option<ConnectionTypeNodeStateData>,
}

impl ConnectionNodeStateData {
    pub fn r#type(&self) -> &Option<ConnectionTypeNodeStateData> {
        &self.r#type
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Default)]
pub struct NodeStateData {
    pub(super) id: Option<NodeId>,
    pub(super) connections: HashMap<NodeConnectionId, ConnectionNodeStateData>,
}

impl NodeStateData {
    pub fn id(&self) -> &Option<NodeId> {
        &self.id
    }

    pub fn connections(&self) -> &HashMap<NodeConnectionId, ConnectionNodeStateData> {
        &self.connections
    }
}
