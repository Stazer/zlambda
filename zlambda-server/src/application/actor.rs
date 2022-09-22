use crate::application::{CreateUdpSocketMessage, CreateUnixListenerMessage};
use crate::udp::UdpSocketActor;
use crate::unix::UnixListenerActor;
use actix::{
    Actor, ActorFutureExt, Addr, Context, Handler, Message, ResponseActFuture, WrapFuture,
};
use tokio::net::ToSocketAddrs;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ApplicationActor {
    udp_socket_actors: Vec<Addr<UdpSocketActor>>,
    unix_listener_actors: Vec<Addr<UnixListenerActor>>,
}

impl Actor for ApplicationActor {
    type Context = Context<Self>;
}

impl<T> Handler<CreateUdpSocketMessage<T>> for ApplicationActor
where
    T: ToSocketAddrs + 'static,
{
    type Result = ResponseActFuture<Self, <CreateUdpSocketMessage<T> as Message>::Result>;

    fn handle(
        &mut self,
        message: CreateUdpSocketMessage<T>,
        _context: &mut <ApplicationActor as Actor>::Context,
    ) -> Self::Result {
        let (socket_address,) = message.into();

        Box::pin(
            async { UdpSocketActor::new(socket_address).await }
                .into_actor(self)
                .map(|result, actor, _context| match result {
                    Ok(address) => {
                        actor.udp_socket_actors.push(address);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }),
        )
    }
}

impl Handler<CreateUnixListenerMessage> for ApplicationActor {
    type Result = ResponseActFuture<Self, <CreateUnixListenerMessage as Message>::Result>;

    fn handle(
        &mut self,
        message: CreateUnixListenerMessage,
        _context: &mut <ApplicationActor as Actor>::Context,
    ) -> Self::Result {
        let (path,) = message.into();

        Box::pin(
            async { UnixListenerActor::new(path).await }
                .into_actor(self)
                .map(|result, actor, _context| match result {
                    Ok(address) => {
                        actor.unix_listener_actors.push(address);
                        Ok(())
                    }
                    Err(e) => Err(e),
                }),
        )
    }
}

impl ApplicationActor {
    pub fn new() -> Addr<Self> {
        (Self {
            udp_socket_actors: Vec::default(),
            unix_listener_actors: Vec::default(),
        })
        .start()
    }
}
