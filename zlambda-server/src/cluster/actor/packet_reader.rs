use crate::algorithm::next_key;
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
use std::io;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct PacketReaderActor {
    id: ConnectionId,
    read_recipient: Recipient<PacketReaderActorReadPacketMessage>,
    remove_recipient: Recipient<NodeActorRemoveConnectionMessage>,
    stream: Addr<TcpStreamActor>,
    buffer: Vec<u8>,
}

impl Actor for PacketReaderActor {
    type Context = Context<Self>;

    #[tracing::instrument]
    fn stopped(&mut self, _: &mut Self::Context) {
        self.stream.do_send(ActorStopMessage);
        self.remove_recipient
            .do_send(NodeActorRemoveConnectionMessage::new(self.id));
    }
}

impl Handler<ActorStopMessage> for PacketReaderActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl Handler<TcpStreamActorReceiveMessage> for PacketReaderActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result,) = message.into();

        let bytes = match result {
            Ok(b) => b,
            Err(e) => {
                error!("{:?}", e);
                context.stop();
                return;
            }
        };

        self.buffer.extend(bytes);

        loop {
            let (read, packet) = match Packet::from_vec(&self.buffer) {
                Ok((r, p)) => (r, p),
                Err(ReadPacketError::UnexpectedEnd) => break,
                Err(e) => {
                    error!("{:?}", e);
                    context.stop();
                    return;
                }
            };

            self.buffer.drain(0..read);

            self.read_recipient
                .do_send(PacketReaderActorReadPacketMessage::new(self.id, packet));
        }
    }
}

impl PacketReaderActor {
    pub fn new(
        id: ConnectionId,
        read_recipient: Recipient<PacketReaderActorReadPacketMessage>,
        remove_recipient: Recipient<NodeActorRemoveConnectionMessage>,
        stream: TcpStream,
    ) -> (Addr<Self>, Addr<TcpStreamActor>) {
        let context = Context::new();
        let stream = TcpStreamActor::new(
            context.address().recipient(),
            Some(context.address().recipient()),
            stream,
        );

        (
            context.run(Self {
                id,
                read_recipient,
                remove_recipient,
                stream: stream.clone(),
                buffer: Vec::default(),
            }),
            stream,
        )
    }
}
