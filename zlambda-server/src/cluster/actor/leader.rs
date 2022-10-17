use crate::cluster::{
    ConnectionId, NodeActorRemoveConnectionMessage, NodeId, Packet, PacketReaderActor,
    PacketReaderActorReadPacketMessage,
};
use crate::common::{
    ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderNodeFollowerActor {}

impl Actor for LeaderNodeFollowerActor {
    type Context = Context<Self>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderNodeConnectionActor {
    id: ConnectionId,
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    packet_reader_actor_address: Addr<PacketReaderActor<()>>,
}

impl Actor for LeaderNodeConnectionActor {
    type Context = Context<Self>;
}

impl Handler<ActorStopMessage> for LeaderNodeConnectionActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _message: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop()
    }
}

impl Handler<PacketReaderActorReadPacketMessage<()>> for LeaderNodeConnectionActor {
    type Result = <PacketReaderActorReadPacketMessage<()> as Message>::Result;

    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage<()>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (_, packet) = message.into();

        /*match packet {
        }*/
    }
}

impl Handler<NodeActorRemoveConnectionMessage<()>> for LeaderNodeConnectionActor {
    type Result = <NodeActorRemoveConnectionMessage<()> as Message>::Result;

    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage<()>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl LeaderNodeConnectionActor {
    fn new(id: ConnectionId, tcp_stream: TcpStream) -> Addr<Self> {
        Self::create(move |context| {
            let (packet_reader_actor_address, tcp_stream_actor_address) = PacketReaderActor::new(
                (),
                context.address().recipient(),
                context.address().recipient(),
                tcp_stream,
            );

            Self {
                id,
                tcp_stream_actor_address,
                packet_reader_actor_address,
            }
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderNodeActor {
    tcp_listener_actor_address: Addr<TcpListenerActor>,
    connection_actor_addresses: HashMap<ConnectionId, Addr<LeaderNodeConnectionActor>>,
    node_actor_addresses: HashMap<NodeId, ()>,
}

impl Actor for LeaderNodeActor {
    type Context = Context<Self>;
}

impl Handler<ActorStopMessage> for LeaderNodeActor {
    type Result = <ActorStopMessage as Message>::Result;

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

    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result,) = message.into();

        match result {
            Ok(tcp_stream) => {}
            Err(error) => {
                error!("{}", error);
            }
        }
    }
}

impl LeaderNodeActor {
    pub fn new(tcp_listener: TcpListener) -> Addr<LeaderNodeActor> {
        Self::create(move |context| Self {
            connection_actor_addresses: HashMap::default(),
            tcp_listener_actor_address: TcpListenerActor::new(
                context.address().recipient(),
                Some(context.address().recipient()),
                tcp_listener,
            ),
            node_actor_addresses: HashMap::<_, ()>::default(),
        })
    }
}
