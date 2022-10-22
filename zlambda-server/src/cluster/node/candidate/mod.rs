use actix::{Actor, Context};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CandidateNodeActor {}

impl Actor for CandidateNodeActor {
    type Context = Context<Self>;
}
