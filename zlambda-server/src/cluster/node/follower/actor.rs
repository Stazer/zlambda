use crate::cluster::{
    NodeActor, RegisteredFollowerNodeActor, RegisteringFollowerNodeActor,
    UnregisteredFollowerNodeActor, UpdateFollowerNodeActorMessage,
};
use crate::common::TcpListenerActor;
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::net::ToSocketAddrs;
use tracing::trace;

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

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: UpdateFollowerNodeActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (actor,) = message.into();
        *self = actor;

        trace!(
            "Change to {} state",
            match self {
                Self::Unregistered(_) => "unregistered",
                Self::Registering(_) => "registering",
                Self::Registered(_) => "registered",
            }
        );
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
