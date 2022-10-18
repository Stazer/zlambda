use crate::cluster::{FollowerNodeActor, RegisteringFollowerNodeActor};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message, WrapFuture};
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RegisterActorMessage {
    address: SocketAddr,
}

impl From<RegisterActorMessage> for (SocketAddr,) {
    fn from(message: RegisterActorMessage) -> Self {
        (message.address,)
    }
}

impl Message for RegisterActorMessage {
    type Result = ();
}

impl RegisterActorMessage {
    pub fn new(address: SocketAddr) -> Self {
        Self { address }
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct UnregisteredFollowerNodeActor {
    follower_node_actor_address: Addr<FollowerNodeActor>,
}

impl Actor for UnregisteredFollowerNodeActor {
    type Context = Context<Self>;
}

impl Handler<RegisterActorMessage> for UnregisteredFollowerNodeActor {
    type Result = <RegisterActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: RegisterActorMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (address,) = message.into();
        let follower_node_actor_address = self.follower_node_actor_address.clone();

        context.wait(
            async move {
                match TcpStream::connect(address).await {
                    Ok(tcp_stream) => RegisteringFollowerNodeActor::new(
                        follower_node_actor_address.clone(),
                        tcp_stream,
                    ),
                    Err(error) => {
                        error!("{}", error);
                        todo!();
                    }
                };
            }
            .into_actor(self),
        );

        context.stop();
    }
}

impl UnregisteredFollowerNodeActor {
    pub fn new(follower_node_actor_address: Addr<FollowerNodeActor>) -> Addr<Self> {
        (Self {
            follower_node_actor_address,
        })
        .start()
    }
}
