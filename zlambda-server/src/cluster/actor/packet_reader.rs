

use crate::cluster::{
    NodeActorRemoveConnectionMessage, Packet, PacketReaderActorReadPacketMessage, ReadPacketError,
};
use crate::common::{
    ActorStopMessage,
    TcpStreamActor, TcpStreamActorReceiveMessage,
};
use actix::{
    Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message, Recipient,
};


use std::fmt::Debug;

use tokio::net::{TcpStream};
use tracing::{error};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct PacketReaderActor<T>
where
    T: Clone + Debug + Send + Sync + Unpin + 'static,
{
    id: T,
    read_recipient: Recipient<PacketReaderActorReadPacketMessage<T>>,
    remove_recipient: Recipient<NodeActorRemoveConnectionMessage<T>>,
    stream: Addr<TcpStreamActor>,
    buffer: Vec<u8>,
}

impl<T> Actor for PacketReaderActor<T>
where
    T: Clone + Debug + Send + Sync + Unpin + 'static,
{
    type Context = Context<Self>;

    #[tracing::instrument]
    fn stopped(&mut self, _: &mut Self::Context) {
        self.stream.do_send(ActorStopMessage);
        self.remove_recipient
            .do_send(NodeActorRemoveConnectionMessage::new(self.id.clone()));
    }
}

impl<T> Handler<ActorStopMessage> for PacketReaderActor<T>
where
    T: Clone + Debug + Send + Sync + Unpin + 'static,
{
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl<T> Handler<TcpStreamActorReceiveMessage> for PacketReaderActor<T>
where
    T: Clone + Debug + Send + Sync + Unpin + 'static,
{
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
                .do_send(PacketReaderActorReadPacketMessage::new(
                    self.id.clone(),
                    packet,
                ));
        }
    }
}

impl<T> PacketReaderActor<T>
where
    T: Clone + Debug + Send + Sync + Unpin + 'static,
{
    pub fn new(
        id: T,
        read_recipient: Recipient<PacketReaderActorReadPacketMessage<T>>,
        remove_recipient: Recipient<NodeActorRemoveConnectionMessage<T>>,
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
