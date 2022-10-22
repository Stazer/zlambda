mod registered;
mod registering;
mod unregistered;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use registered::*;
pub use registering::*;
pub use unregistered::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::cluster::NodeActor;
use crate::common::TcpListenerActor;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use std::fmt::Debug;
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
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (actor,) = message.into();
        *self = actor;
    }
}

impl FollowerNodeActor {
    pub fn new<T>(
        registration_address: T,
        node_actor_address: Addr<NodeActor>,
        tcp_listener_actor_address: Addr<TcpListenerActor>,
        tcp_listener_socket_local_address: SocketAddr,
    ) -> Addr<Self>
    where
        T: ToSocketAddrs + Send + Sync + Debug + 'static,
    {
        Self::create(move |context| {
            Self::Unregistered(UnregisteredFollowerNodeActor::new(
                registration_address,
                node_actor_address,
                context.address(),
                tcp_listener_actor_address,
                tcp_listener_socket_local_address,
            ))
        })
    }
}