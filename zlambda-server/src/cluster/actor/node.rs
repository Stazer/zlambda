use crate::algorithm::next_key;
use crate::cluster::actor::{FollowerNodeActor, LeaderNodeActor, PacketReaderActor};
use crate::cluster::NodeId;
use crate::cluster::{
    ConnectionId, NodeActorRegisterMessage, NodeActorRemoveConnectionMessage,
    NodeRegisterResponsePacketError, Packet, PacketReaderActorReadPacketMessage, ReadPacketError,
};
use crate::common::{
    ActorExecuteMessage, ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage,
    TcpStreamActor, TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, Recipient,
    ResponseActFuture, WrapFuture,
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::io;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeActor {
    Leader(Addr<LeaderNodeActor>),
    Follower(Addr<FollowerNodeActor>),
}

impl Actor for NodeActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _context: &mut Self::Context) {
        match self {
            Self::Leader(leader) => {
                leader.do_send(ActorStopMessage);
            }
            Self::Follower(follower) => {
                follower.do_send(ActorStopMessage);
            }
        }
    }
}

impl Handler<ActorStopMessage> for NodeActor {
    type Result = <ActorStopMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(&mut self, _message: ActorStopMessage, context: &mut <Self as Actor>::Context) {
        context.stop();
    }
}

impl NodeActor {
    pub async fn new<S, T>(
        listener_address: S,
        leader_stream_address: Option<T>,
    ) -> Result<Addr<Self>, io::Error>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs + Debug + Display + Send + Sync + 'static,
    {
        let listener = TcpListener::bind(listener_address).await?;

        match leader_stream_address {
            Some(l) => Ok(Self::create(move |context| {
                Self::Follower(FollowerNodeActor::new(context.address(), listener, l))
            })),
            None => Ok(Self::create(|context| {
                Self::Leader(LeaderNodeActor::new(context.address(), listener))
            })),
        }
    }
}
