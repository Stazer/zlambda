use crate::cluster::FollowerNodeActor;
use actix::Message;
use std::fmt::Debug;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct UpdateFollowerNodeActorMessage {
    actor: FollowerNodeActor,
}

impl From<UpdateFollowerNodeActorMessage> for (FollowerNodeActor,) {
    fn from(message: UpdateFollowerNodeActorMessage) -> Self {
        (message.actor,)
    }
}

impl Message for UpdateFollowerNodeActorMessage {
    type Result = ();
}

impl UpdateFollowerNodeActorMessage {
    pub fn new(actor: FollowerNodeActor) -> Self {
        Self { actor }
    }

    pub fn actor(&self) -> &FollowerNodeActor {
        &self.actor
    }
}
