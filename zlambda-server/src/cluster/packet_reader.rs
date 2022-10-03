use crate::cluster::{
    NodeActor, NodeActorRemoveConnectionMessage, NodeConnectionId, Packet, ReadPacketError,
};
use crate::common::{ActorStopMessage, TcpStreamActor, TcpStreamActorReceiveMessage};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use tokio::net::TcpStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct PacketReaderActorReadPacketMessage {
    id: NodeConnectionId,
    packet: Packet,
}

impl From<PacketReaderActorReadPacketMessage> for (NodeConnectionId, Packet) {
    fn from(message: PacketReaderActorReadPacketMessage) -> Self {
        (message.id, message.packet)
    }
}

impl Message for PacketReaderActorReadPacketMessage {
    type Result = ();
}

impl PacketReaderActorReadPacketMessage {
    pub fn new(id: NodeConnectionId, packet: Packet) -> Self {
        Self { id, packet }
    }

    pub fn id(&self) -> NodeConnectionId {
        self.id
    }

    pub fn packet(&self) -> &Packet {
        &self.packet
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct PacketReaderActor {
    id: NodeConnectionId,
    recipient: Addr<NodeActor>,
    stream: Addr<TcpStreamActor>,
    buffer: Vec<u8>,
}

impl Actor for PacketReaderActor {
    type Context = Context<Self>;

    #[tracing::instrument]
    fn stopped(&mut self, _: &mut Self::Context) {
        self.stream.do_send(ActorStopMessage);
        self.recipient
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
                eprintln!("{:?}", e);
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
                    eprintln!("{:?}", e);
                    context.stop();
                    return;
                }
            };

            self.buffer.drain(0..read);

            self.recipient
                .do_send(PacketReaderActorReadPacketMessage::new(self.id, packet));
        }
    }
}

impl PacketReaderActor {
    pub fn new(
        id: NodeConnectionId,
        recipient: Addr<NodeActor>,
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
                recipient,
                stream: stream.clone(),
                buffer: Vec::default(),
            }),
            stream,
        )
    }
}
