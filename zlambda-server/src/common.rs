use actix::{
    Actor, ActorContext, Addr, AsyncContext, AtomicResponse, Context, Handler, Message, Recipient,
    StreamHandler, WrapFuture,
};
use bytes::Bytes;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::io::ReaderStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ActorStopMessage;

impl Message for ActorStopMessage {
    type Result = ();
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TcpStreamActorReceiveMessage {
    result: Result<Bytes, io::Error>,
}

impl From<TcpStreamActorReceiveMessage> for (Result<Bytes, io::Error>,) {
    fn from(message: TcpStreamActorReceiveMessage) -> (Result<Bytes, io::Error>,) {
        (message.result,)
    }
}

impl Message for TcpStreamActorReceiveMessage {
    type Result = ();
}

impl TcpStreamActorReceiveMessage {
    pub fn new(result: Result<Bytes, io::Error>) -> Self {
        Self { result }
    }

    pub fn result(&self) -> &Result<Bytes, io::Error> {
        &self.result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TcpStreamActorSendMessage {
    data: Bytes,
}

impl From<TcpStreamActorSendMessage> for (Bytes,) {
    fn from(message: TcpStreamActorSendMessage) -> Self {
        (message.data,)
    }
}

impl Message for TcpStreamActorSendMessage {
    type Result = Result<(), io::Error>;
}

impl TcpStreamActorSendMessage {
    pub fn new(data: Bytes) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &Bytes {
        &self.data
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TcpStreamActor {
    receive_recipient: Recipient<TcpStreamActorReceiveMessage>,
    stop_recipient: Option<Recipient<ActorStopMessage>>,
    owned_write_half: Arc<Mutex<OwnedWriteHalf>>,
}

impl Actor for TcpStreamActor {
    type Context = Context<Self>;

    fn stopped(&mut self, context: &mut Self::Context) {
        if let Some(stop_recipient) = &self.stop_recipient {
            stop_recipient.do_send(ActorStopMessage);
        }
    }
}

impl Handler<ActorStopMessage> for TcpStreamActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl Handler<TcpStreamActorSendMessage> for TcpStreamActor {
    type Result = AtomicResponse<Self, <TcpStreamActorSendMessage as Message>::Result>;

    fn handle(
        &mut self,
        message: TcpStreamActorSendMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let owned_write_half = self.owned_write_half.clone();

        AtomicResponse::new(Box::pin(
            async move {
                let mut writer = match owned_write_half.lock() {
                    Ok(w) => w,
                    Err(_) => todo!(),
                };

                match writer.write(message.data()).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            .into_actor(self),
        ))
    }
}

impl StreamHandler<Result<Bytes, io::Error>> for TcpStreamActor {
    fn handle(&mut self, item: Result<Bytes, io::Error>, _context: &mut <Self as Actor>::Context) {
        self.receive_recipient
            .do_send(TcpStreamActorReceiveMessage::new(item));
    }
}

impl TcpStreamActor {
    pub fn new(
        receive_recipient: Recipient<TcpStreamActorReceiveMessage>,
        stop_recipient: Option<Recipient<ActorStopMessage>>,
        tcp_stream: TcpStream,
    ) -> Addr<Self> {
        Self::create(move |context| {
            let (owned_read_half, owned_write_half) = tcp_stream.into_split();

            context.add_stream(ReaderStream::new(owned_read_half));

            Self {
                receive_recipient,
                stop_recipient,
                owned_write_half: Arc::new(Mutex::new(owned_write_half)),
            }
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TcpListenerActorAcceptMessage {
    result: Result<TcpStream, io::Error>,
}

impl From<TcpListenerActorAcceptMessage> for (Result<TcpStream, io::Error>,) {
    fn from(message: TcpListenerActorAcceptMessage) -> Self {
        (message.result,)
    }
}

impl Message for TcpListenerActorAcceptMessage {
    type Result = ();
}

impl TcpListenerActorAcceptMessage {
    pub fn new(result: Result<TcpStream, io::Error>) -> Self {
        Self { result }
    }

    pub fn result(&self) -> &Result<TcpStream, io::Error> {
        &self.result
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TcpListenerActor {
    accept_recipient: Recipient<TcpListenerActorAcceptMessage>,
    stop_recipient: Option<Recipient<ActorStopMessage>>,
}

impl Actor for TcpListenerActor {
    type Context = Context<Self>;

    fn stopped(&mut self, context: &mut Self::Context) {
        if let Some(stop_recipient) = &self.stop_recipient {
            stop_recipient.do_send(ActorStopMessage);
        }
    }
}

impl Handler<ActorStopMessage> for TcpListenerActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl StreamHandler<Result<TcpStream, io::Error>> for TcpListenerActor {
    fn handle(
        &mut self,
        item: Result<TcpStream, io::Error>,
        _context: &mut <Self as Actor>::Context,
    ) {
        self.accept_recipient
            .do_send(TcpListenerActorAcceptMessage::new(item));
    }
}

impl TcpListenerActor {
    pub fn new(
        accept_recipient: Recipient<TcpListenerActorAcceptMessage>,
        stop_recipient: Option<Recipient<ActorStopMessage>>,
        tcp_listener: TcpListener,
    ) -> Addr<Self> {
        Self::create(|context| {
            context.add_stream(TcpListenerStream::new(tcp_listener));

            Self {
                accept_recipient,
                stop_recipient,
            }
        })
    }
}
