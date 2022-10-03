use crate::algorithm::next_key;
use crate::cluster::{
    ConnectionNodeStateData, ConnectionTypeNodeStateData, NodeConnectionId, NodeId, NodeStateData,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait NodeStateTransition {
    type Result = ();

    fn apply(self, data: &mut NodeStateData) -> Self::Result;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SetIdNodeStateTransition {
    id: NodeId,
}

impl SetIdNodeStateTransition {
    pub fn new(id: NodeId) -> Self {
        Self { id }
    }
}

impl NodeStateTransition for SetIdNodeStateTransition {
    fn apply(self, data: &mut NodeStateData) {
        assert!(data.id.is_none());
        data.id = Some(self.id);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct CreateConnectionNodeStateTransition;

impl NodeStateTransition for CreateConnectionNodeStateTransition {
    type Result = NodeConnectionId;

    fn apply(self, data: &mut NodeStateData) -> Self::Result {
        let id = next_key(&data.connections);

        data.connections
            .insert(id, ConnectionNodeStateData { r#type: None });

        id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct DestroyConnectionNodeStateTransition {
    id: NodeConnectionId,
}

impl DestroyConnectionNodeStateTransition {
    pub fn new(id: NodeConnectionId) -> Self {
        Self { id }
    }
}

impl NodeStateTransition for DestroyConnectionNodeStateTransition {
    fn apply(self, data: &mut NodeStateData) -> Self::Result {
        assert!(data.connections.remove(&self.id).is_some());
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct RegisterNodeNodeStateTransition;

impl NodeStateTransition for RegisterNodeNodeStateTransition {
    type Result = NodeConnectionId;

    fn apply(self, data: &mut NodeStateData) -> Self::Result {
        assert!(data.id.is_none());
        let id = next_key(&data.connections);

        assert!(data
            .connections
            .insert(
                id,
                ConnectionNodeStateData {
                    r#type: Some(ConnectionTypeNodeStateData::Node),
                }
            )
            .is_none());

        id
    }
}
