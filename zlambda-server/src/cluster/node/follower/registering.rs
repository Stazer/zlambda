use crate::cluster::{FollowerNodeActor, PacketReader};
use crate::common::{TcpStreamActor, TcpStreamActorReceiveMessage};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use tokio::net::TcpStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RegisteringFollowerNodeActor {
    follower_node_actor_address: Addr<FollowerNodeActor>,
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    packet_reader: PacketReader,
}

impl Actor for RegisteringFollowerNodeActor {
    type Context = Context<Self>;
}

impl Handler<TcpStreamActorReceiveMessage> for RegisteringFollowerNodeActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        println!("Hello World");
    }
}

impl RegisteringFollowerNodeActor {
    pub fn new(
        follower_node_actor_address: Addr<FollowerNodeActor>,

        tcp_stream: TcpStream,
    ) -> Addr<Self> {
        Self::create(move |context| Self {
            follower_node_actor_address,
            tcp_stream_actor_address: TcpStreamActor::new(
                context.address().recipient(),
                None,
                tcp_stream,
            ),
            packet_reader: PacketReader::default(),
        })
    }
}
