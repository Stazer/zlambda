use crate::cluster::{
    FollowerNodeActor, NodeActor, RegisteringFollowerNodeActor, UpdateFollowerNodeActorMessage,
};
use crate::common::TcpListenerActor;
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, WrapFuture,
};
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RegisterActorMessage<T>
where
    T: ToSocketAddrs + Send + Sync + Debug + 'static,
{
    address: T,
}

impl<T> From<RegisterActorMessage<T>> for (T,)
where
    T: ToSocketAddrs + Send + Sync + Debug + 'static,
{
    fn from(message: RegisterActorMessage<T>) -> Self {
        (message.address,)
    }
}

impl<T> Message for RegisterActorMessage<T>
where
    T: ToSocketAddrs + Send + Sync + Debug + 'static,
{
    type Result = ();
}

impl<T> RegisterActorMessage<T>
where
    T: ToSocketAddrs + Send + Sync + Debug + 'static,
{
    pub fn new(address: T) -> Self {
        Self { address }
    }

    pub fn address(&self) -> &T {
        &self.address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct UnregisteredFollowerNodeActor {
    node_actor_address: Addr<NodeActor>,
    follower_node_actor_address: Addr<FollowerNodeActor>,
    tcp_listener_actor_address: Addr<TcpListenerActor>,
    tcp_listener_socket_local_address: SocketAddr,
}

impl Actor for UnregisteredFollowerNodeActor {
    type Context = Context<Self>;
}

impl<T> Handler<RegisterActorMessage<T>> for UnregisteredFollowerNodeActor
where
    T: ToSocketAddrs + Send + Sync + Debug + 'static,
{
    type Result = <RegisterActorMessage<T> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: RegisterActorMessage<T>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (address,) = message.into();
        let node_actor_address = self.node_actor_address.clone();
        let follower_node_actor_address = self.follower_node_actor_address.clone();
        let tcp_listener_actor_address = self.tcp_listener_actor_address.clone();

        trace!("Connect to {:?}", &address);

        context.wait(
            async move { TcpStream::connect(address).await }
                .into_actor(self)
                .map(move |result, actor, context| {
                    let tcp_stream = match result {
                        Ok(tcp_stream) => tcp_stream,
                        Err(e) => {
                            error!("{}", e);
                            context.stop();

                            return;
                        }
                    };

                    match follower_node_actor_address.try_send(UpdateFollowerNodeActorMessage::new(
                        FollowerNodeActor::Registering(RegisteringFollowerNodeActor::new(
                            node_actor_address,
                            follower_node_actor_address.clone(),
                            tcp_listener_actor_address,
                            actor.tcp_listener_socket_local_address,
                            tcp_stream,
                        )),
                    )) {
                        Ok(()) => {}
                        Err(e) => {
                            error!("{}", e);
                            context.stop();
                        }
                    }
                }),
        );
    }
}

impl UnregisteredFollowerNodeActor {
    pub fn new<T>(
        registration_address: T,
        node_actor_address: Addr<NodeActor>,
        follower_node_actor_address: Addr<FollowerNodeActor>,
        tcp_listener_actor_address: Addr<TcpListenerActor>,
        tcp_listener_socket_local_address: SocketAddr,
    ) -> Addr<Self>
    where
        T: ToSocketAddrs + Send + Sync + Debug + 'static,
    {
        let address = (Self {
            node_actor_address,
            follower_node_actor_address,
            tcp_listener_actor_address,
            tcp_listener_socket_local_address,
        })
        .start();

        address.do_send(RegisterActorMessage::new(registration_address));

        address
    }
}
