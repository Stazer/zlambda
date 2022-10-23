mod candidate;
mod follower;
mod leader;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use candidate::*;
pub use follower::*;
pub use leader::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common::{TcpListenerActor, TcpListenerActorAcceptMessage};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use std::fmt::{Debug, Display};
use std::io;
use tokio::net::{TcpListener, ToSocketAddrs};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum NodeActor {
    Leader(Addr<LeaderNodeActor>),
    Follower(Addr<FollowerNodeActor>),
    Candidate(Addr<CandidateNodeActor>),
}

impl Actor for NodeActor {
    type Context = Context<Self>;
}

impl Handler<TcpListenerActorAcceptMessage> for NodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    fn handle(
        &mut self,
        _message: TcpListenerActorAcceptMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl NodeActor {
    pub async fn new<S, T>(
        listener_address: S,
        registration_address: Option<T>,
    ) -> Result<Addr<Self>, io::Error>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs + Debug + Display + Send + Sync + 'static,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;
        let tcp_listener_socket_local_address = tcp_listener.local_addr()?;

        Ok(Self::create(move |context| {
            let tcp_listener_actor_address =
                TcpListenerActor::new(context.address().recipient(), tcp_listener);

            match registration_address {
                Some(registration_address) => Self::Follower(FollowerNodeActor::new(
                    registration_address,
                    context.address(),
                    tcp_listener_actor_address,
                    tcp_listener_socket_local_address,
                )),
                None => Self::Leader(LeaderNodeActor::new(
                    context.address(),
                    tcp_listener_actor_address,
                    0,
                    0,
                    [(0, tcp_listener_socket_local_address)].into(),
                )),
            }
        }))
    }
}
