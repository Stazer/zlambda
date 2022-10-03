use crate::cluster::{NodeId, ConnectionId, StateData};
use actix::Message;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct StateActorCreateConnectionMessage {}

impl Message for StateActorCreateConnectionMessage {
    type Result = ConnectionId;
}

impl StateActorCreateConnectionMessage {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct StateActorDestroyConnectionMessage {
    connection_id: ConnectionId,
}

impl From<StateActorDestroyConnectionMessage> for (ConnectionId,) {
    fn from(message: StateActorDestroyConnectionMessage) -> Self {
        (message.connection_id,)
    }
}

impl Message for StateActorDestroyConnectionMessage {
    type Result = ();
}

impl StateActorDestroyConnectionMessage {
    pub fn new(connection_id: ConnectionId) -> Self {
        Self { connection_id }
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct StateActorReadDataMessage<F, R>
where
    F: FnOnce(&StateData) -> R,
    R: 'static,
{
    function: F,
}

impl<F, R> From<StateActorReadDataMessage<F, R>> for (F,)
where
    F: FnOnce(&StateData) -> R,
    R: 'static,
{
    fn from(message: StateActorReadDataMessage<F, R>) -> Self {
        (message.function,)
    }
}

impl<F, R> Message for StateActorReadDataMessage<F, R>
where
    F: FnOnce(&StateData) -> R,
    R: 'static,
{
    type Result = R;
}

impl<F, R> StateActorReadDataMessage<F, R>
where
    F: FnOnce(&StateData) -> R,
    R: 'static,
{
    pub fn new(function: F) -> Self {
        Self { function }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct StateActorCreateNodeConnectionMessage {
    node_id: NodeId,
}

impl From<StateActorCreateNodeConnectionMessage> for (NodeId, ) {
    fn from(message: StateActorCreateNodeConnectionMessage) -> Self {
        (message.node_id, )
    }
}

impl Message for StateActorCreateNodeConnectionMessage {
    type Result = ConnectionId;
}

impl StateActorCreateNodeConnectionMessage {
    pub fn new(node_id: NodeId) -> Self {
        Self {
            node_id,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}
