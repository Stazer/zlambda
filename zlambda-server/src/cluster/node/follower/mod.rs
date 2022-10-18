mod registered;
mod registering;
mod unregistered;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use registered::*;
pub use registering::*;
pub use unregistered::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message, WrapFuture};
use std::net::SocketAddr;
use tokio::net::ToSocketAddrs;

////////////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum FollowerNodeActor {
    Unregistered(Addr<UnregisteredFollowerNodeActor>),
    Registering(Addr<RegisteringFollowerNodeActor>),
    Registered(Addr<RegisteredFollowerNodeActor>),
}

impl Actor for FollowerNodeActor {
    type Context = Context<Self>;
}

impl Handler<UpdateFollowerNodeActorMessage> for FollowerNodeActor {
    type Result = <UpdateFollowerNodeActorMessage as Message>::Result;

    fn handle(
        &mut self,
        message: UpdateFollowerNodeActorMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (actor,) = message.into();
        *self = actor;
    }
}

impl FollowerNodeActor {
    pub fn new<T>(leader_address: T) -> Addr<Self>
    where
        T: ToSocketAddrs,
    {
        Self::create(move |context| {
            let actor = UnregisteredFollowerNodeActor::new(context.address());

            Self::Unregistered(actor)
        })
    }
}
