mod follower;
mod connection;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use connection::*;
pub use follower::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::cluster::{
    ConnectionId, NodeActorRemoveConnectionMessage, NodeId, Packet, PacketReaderActor,
    PacketReaderActorReadPacketMessage, PacketReader,
};
use crate::common::{
    ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor,
    TcpStreamActorReceiveMessage,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeActor {
    tcp_listener_actor_address: Addr<TcpListenerActor>,
    connection_actor_addresses: HashMap<ConnectionId, ()>,
    node_actor_addresses: HashMap<NodeId, ()>,
}

impl Actor for LeaderNodeActor {
    type Context = Context<Self>;
}

impl Handler<ActorStopMessage> for LeaderNodeActor {
    type Result = <ActorStopMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop()
    }
}

impl Handler<TcpListenerActorAcceptMessage> for LeaderNodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result,) = message.into();

        match result {
            Ok(tcp_stream) => {

            }
            Err(error) => {
                error!("{}", error);
            }
        }
    }
}

impl LeaderNodeActor {
    pub fn new(tcp_listener: TcpListener) -> Addr<LeaderNodeActor> {
        Self::create(move |context| Self {
            connection_actor_addresses: HashMap::<_, ()>::default(),
            tcp_listener_actor_address: TcpListenerActor::new(
                context.address().recipient(),
                Some(context.address().recipient()),
                tcp_listener,
            ),
            node_actor_addresses: HashMap::<_, ()>::default(),
        })
    }
}
