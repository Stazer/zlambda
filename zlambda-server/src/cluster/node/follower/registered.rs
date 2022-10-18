use crate::cluster::PacketReader;
use crate::common::{TcpStreamActor, TcpStreamActorReceiveMessage};
use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use tokio::net::TcpStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RegisteredFollowerNodeActor {
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    packet_reader: PacketReader,
}

impl Actor for RegisteredFollowerNodeActor {
    type Context = Context<Self>;
}

impl Handler<TcpStreamActorReceiveMessage> for RegisteredFollowerNodeActor {
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

impl RegisteredFollowerNodeActor {
    pub fn new(tcp_stream: TcpStream) -> Addr<Self> {
        Self::create(move |context| Self {
            tcp_stream_actor_address: TcpStreamActor::new(
                context.address().recipient(),
                None,
                tcp_stream,
            ),
            packet_reader: PacketReader::default(),
        })
    }
}
