use crate::algorithm::next_key;
use crate::cluster::{
    ConnectionData, ConnectionId, ConnectionTypeData, StateActorCreateConnectionMessage,
    StateActorDestroyConnectionMessage, StateActorReadDataMessage, StateData,
    StateActorCreateNodeConnectionMessage, NodeId,
};
use crate::common::ActorStopMessage;
use actix::dev::MessageResponse;
use actix::{Actor, ActorContext, Addr, Context, Handler, Message};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct StateActor {
    data: StateData,
}

impl Actor for StateActor {
    type Context = Context<Self>;
}

impl Handler<ActorStopMessage> for StateActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl Handler<StateActorCreateConnectionMessage> for StateActor {
    type Result = <StateActorCreateConnectionMessage as Message>::Result;

    fn handle(
        &mut self,
        _: StateActorCreateConnectionMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let id = next_key(&self.data.connections());

        assert!(self
            .data
            .connections
            .insert(id, ConnectionData::new(id, None))
            .is_none());

        id
    }
}

impl Handler<StateActorDestroyConnectionMessage> for StateActor {
    type Result = <StateActorDestroyConnectionMessage as Message>::Result;

    fn handle(
        &mut self,
        message: StateActorDestroyConnectionMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (connection_id,): (ConnectionId,) = message.into();

        let connection = self
            .data
            .connections()
            .get(&connection_id)
            .expect("Connection not found");
        match *connection.r#type() {
            None => {}
            Some(ConnectionTypeData::Node { node_id }) => {
                assert!(self.data.nodes.remove(&node_id).is_some());
            }
            Some(ConnectionTypeData::Client { client_id }) => {
                assert!(self.data.clients.remove(&client_id).is_some());
            }
        }

        assert!(self.data.connections.remove(&connection_id).is_some());
    }
}

impl<F, R> Handler<StateActorReadDataMessage<F, R>> for StateActor
where
    F: FnOnce(&StateData) -> R,
    R: MessageResponse<Self, StateActorReadDataMessage<F, R>> + 'static,
{
    type Result = <StateActorReadDataMessage<F, R> as Message>::Result;

    fn handle(
        &mut self,
        message: StateActorReadDataMessage<F, R>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (function,): (F,) = message.into();

        function(&self.data)
    }
}

impl Handler<StateActorCreateNodeConnectionMessage> for StateActor {
    type Result = <StateActorCreateNodeConnectionMessage as  Message>::Result;

    fn handle(
        &mut self,
        message: StateActorCreateNodeConnectionMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let connection_id = next_key(&self.data.connections());

        assert!(self
            .data
            .connections
            .insert(connection_id, ConnectionData::new(connection_id, Some(ConnectionTypeData::Node {
                node_id: message.node_id(),
            })))
            .is_none());

        self.data.nodes.get_mut(&message.node_id()).expect("Node not found").connection_id = Some(connection_id);

        connection_id
    }
}

impl StateActor {
    pub fn new() -> Addr<Self> {
        (Self {
            data: StateData::default(),
        })
        .start()
    }
}
