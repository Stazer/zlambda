use actix::{
    Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message, Recipient,
    ResponseActFuture, StreamHandler, WrapFuture,
};
use bytes::Bytes;
use std::error::Error;
use std::fmt::Debug;
use std::io;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::io::ReaderStream;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct StopActorMessage;

impl Message for StopActorMessage {
    type Result = ();
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct TcpStreamActor {
    recipient: Recipient<TcpStreamActorReceiveMessage>,
    owned_write_half: Arc<Mutex<OwnedWriteHalf>>,
}

impl Actor for TcpStreamActor {
    type Context = Context<Self>;
}

impl Handler<TcpStreamActorSendMessage> for TcpStreamActor {
    type Result = ResponseActFuture<Self, <TcpStreamActorSendMessage as Message>::Result>;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorSendMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let owned_write_half = self.owned_write_half.clone();

        Box::pin(
            async move {
                owned_write_half
                    .lock()
                    .await
                    .write(message.data())
                    .await
                    .map(|_bytes| ())
            }
            .into_actor(self),
        )
    }
}

impl Handler<StopActorMessage> for TcpStreamActor {
    type Result = <StopActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: StopActorMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl Handler<UpdateRecipientActorMessage<TcpStreamActorReceiveMessage>> for TcpStreamActor {
    type Result = <UpdateRecipientActorMessage<TcpStreamActorReceiveMessage> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: UpdateRecipientActorMessage<TcpStreamActorReceiveMessage>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        (self.recipient,) = message.into();
    }
}

impl StreamHandler<Result<Bytes, io::Error>> for TcpStreamActor {
    #[tracing::instrument]
    fn handle(&mut self, item: Result<Bytes, io::Error>, context: &mut <Self as Actor>::Context) {
        let recipient = self.recipient.clone();

        context.wait(
            async move {
                recipient
                    .send(TcpStreamActorReceiveMessage::new(item))
                    .await
                    .expect("Cannot send TcpStreamActorReceiveMessage");
            }
            .into_actor(self),
        );
    }
}

impl TcpStreamActor {
    pub fn new(
        recipient: Recipient<TcpStreamActorReceiveMessage>,
        tcp_stream: TcpStream,
    ) -> Addr<Self> {
        Self::create(move |context| {
            let (owned_read_half, owned_write_half) = tcp_stream.into_split();

            context.add_stream(ReaderStream::new(owned_read_half));

            Self {
                recipient,
                owned_write_half: Arc::new(Mutex::new(owned_write_half)),
            }
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
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

#[derive(Debug)]
pub struct UpdateRecipientActorMessage<T>
where
    T: Message + Send + 'static,
    T::Result: Send + 'static,
{
    recipient: Recipient<T>,
}

impl<T> From<UpdateRecipientActorMessage<T>> for (Recipient<T>,)
where
    T: Message + Send + 'static,
    T::Result: Send + 'static,
{
    fn from(message: UpdateRecipientActorMessage<T>) -> Self {
        (message.recipient,)
    }
}

impl<T> Message for UpdateRecipientActorMessage<T>
where
    T: Message + Send + 'static,
    T::Result: Send + 'static,
{
    type Result = T::Result;
}

impl<T> UpdateRecipientActorMessage<T>
where
    T: Message + Send + 'static,
    T::Result: Send + 'static,
{
    pub fn new(recipient: Recipient<T>) -> Self {
        Self { recipient }
    }

    pub fn recipient(&self) -> &Recipient<T> {
        &self.recipient
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct TcpListenerActor {
    recipient: Recipient<TcpListenerActorAcceptMessage>,
}

impl Actor for TcpListenerActor {
    type Context = Context<Self>;
}

impl Handler<UpdateRecipientActorMessage<TcpListenerActorAcceptMessage>> for TcpListenerActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: UpdateRecipientActorMessage<TcpListenerActorAcceptMessage>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        (self.recipient,) = message.into();
    }
}

impl StreamHandler<Result<TcpStream, io::Error>> for TcpListenerActor {
    #[tracing::instrument]
    fn handle(
        &mut self,
        item: Result<TcpStream, io::Error>,
        _context: &mut <Self as Actor>::Context,
    ) {
        self.recipient
            .do_send(TcpListenerActorAcceptMessage::new(item));
    }
}

impl TcpListenerActor {
    pub fn new(
        recipient: Recipient<TcpListenerActorAcceptMessage>,
        tcp_listener: TcpListener,
    ) -> Addr<Self> {
        Self::create(|context| {
            context.add_stream(TcpListenerStream::new(tcp_listener));

            Self { recipient }
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct UnrecoverableErrorHandler;

impl UnrecoverableErrorHandler {
    pub fn handle<A, E>(error: E, context: &mut A::Context)
    where
        A: Actor,
        E: Error + 'static,
    {
        error!("{}", error);
        context.stop();
    }
}
