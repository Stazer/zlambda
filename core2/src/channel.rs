use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};
use std::fmt::Debug;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait DoSend<T> {
    async fn do_send(self, value: T);
}

#[async_trait]
impl<T> DoSend<T> for oneshot::Sender<T>
where
    T: std::fmt::Debug + Send,
{
    async fn do_send(self, value: T) {
        self.send(value).expect("Data must be received")
    }
}

#[async_trait]
impl<T> DoSend<T> for &mpsc::Sender<T>
where
    T: std::fmt::Debug + Send,
{
    async fn do_send(self, value: T) {
        self.send(value).await.expect("Data must be received")
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[async_trait]
pub trait DoReceive<T> {
    async fn do_receive(self) -> T;
}

#[async_trait]
impl<T> DoReceive<T> for oneshot::Receiver<T>
where
    T: std::fmt::Debug + Send,
{
    async fn do_receive(self) -> T {
        self.await.expect("Data must be received")
    }
}

#[async_trait]
impl<T> DoReceive<T> for &mut mpsc::Receiver<T>
where
    T: std::fmt::Debug + Send,
{
    async fn do_receive(self) -> T {
        self.recv().await.expect("Data must be received")
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AsynchronousMessage<I> {
    input: I,
}

impl<I> From<AsynchronousMessage<I>> for (I,) {
    fn from(message: AsynchronousMessage<I>) -> Self {
        (message.input,)
    }
}

impl<I> AsynchronousMessage<I> {
    pub fn new(input: I) -> Self {
        Self {
            input
        }
    }

    pub fn input(&self) -> &I {
        &self.input
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SynchronousMessage<I, O> {
    input: I,
    sender: oneshot::Sender<O>,
}

impl<I, O> From<SynchronousMessage<I, O>> for (I, oneshot::Sender<O>) {
    fn from(message: SynchronousMessage<I, O>) -> Self {
        (message.input, message.sender)
    }
}

impl<I, O> SynchronousMessage<I, O> {
    pub fn new(input: I, sender: oneshot::Sender<O>) -> Self {
        Self {
            input,
            sender,
        }
    }

    pub fn input(&self) -> &I {
        &self.input
    }

    pub fn sender(&self) -> &oneshot::Sender<O> {
        &self.sender
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct MessageSender<T>
where
    T: Debug + Send
{
    sender: mpsc::Sender<T>
}

impl<T> MessageSender<T>
where
    T: Debug + Send
{
    fn new(
        sender: mpsc::Sender<T>,
    ) -> Self {
        Self {
            sender,
        }
    }

    pub async fn do_send(&self, message: T) {
        self.sender.do_send(message).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MessageReceiver<T>
where
    T: Debug + Send
{
    receiver: mpsc::Receiver<T>
}

impl<T> MessageReceiver<T>
where
    T: Debug + Send
{
    fn new(
        receiver: mpsc::Receiver<T>,
    ) -> Self {
        Self {
            receiver,
        }
    }

    pub async fn do_receive(&mut self) -> T {
        self.receiver.do_receive().await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn message_channel<T>() -> (MessageSender<T>, MessageReceiver<T>)
where
    T: Debug + Send
{
    let (sender, receiver) = mpsc::channel(16);

    (MessageSender::new(sender), MessageReceiver::new(receiver))
}
