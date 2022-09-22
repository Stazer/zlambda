use actix::{Addr, Actor, SyncContext, SyncArbiter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct OperationRequestHandlerActor {
}

impl Actor for OperationRequestHandlerActor {
    type Context = SyncContext<Self>;
}

impl OperationRequestHandlerActor {
    pub fn new() -> Addr<Self> {
        SyncArbiter::start(0, || OperationRequestHandlerActor{})
    }
}
