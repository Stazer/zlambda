use std::fmt::Debug;
use tokio::sync::{mpsc, oneshot};

////////////////////////////////////////////////////////////////////////////////////////////////////

trait DoSend<T> {
    async fn do_send(self, value: T);
}

impl<T> DoSend<T> for oneshot::Sender<T>
where
    T: std::fmt::Debug + Send,
{
    async fn do_send(self, value: T) {
        self.send(value).expect("Data must be received")
    }
}

impl<T> DoSend<T> for &mpsc::Sender<T>
where
    T: std::fmt::Debug + Send,
{
    async fn do_send(self, value: T) {
        self.send(value).await.expect("Data must be received")
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

trait DoReceive<T> {
    async fn do_receive(self) -> T;
}

impl<T> DoReceive<T> for oneshot::Receiver<T>
where
    T: std::fmt::Debug + Send,
{
    async fn do_receive(self) -> T {
        self.await.expect("Data must be received")
    }
}

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
        Self { input }
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
        Self { input, sender }
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
pub struct MessageChannelWriter<T>
where
    T: Debug + Send,
{
    sender: mpsc::Sender<T>,
}

impl<T> MessageChannelWriter<T>
where
    T: Debug + Send,
{
    fn new(sender: mpsc::Sender<T>) -> Self {
        Self { sender }
    }

    pub async fn write_asynchronous<I>(&mut self, input: I)
    where
        T: From<AsynchronousMessage<I>>,
    {
        self.sender
            .do_send(T::from(AsynchronousMessage::new(input)))
            .await;
    }

    pub async fn write_synchronous<I, O>(&mut self, input: I) -> O
    where
        I: Debug + Send,
        O: Debug + Send,
        T: From<SynchronousMessage<I, O>>,
    {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(T::from(SynchronousMessage::new(input, sender)))
            .await;

        receiver.do_receive().await
    }

    pub async fn do_write(&self, message: T) {
        self.sender.send(message).await.expect("")
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MessageChannelReader<T>
where
    T: Debug + Send,
{
    receiver: mpsc::Receiver<T>,
}

impl<T> MessageChannelReader<T>
where
    T: Debug + Send,
{
    fn new(receiver: mpsc::Receiver<T>) -> Self {
        Self { receiver }
    }

    pub async fn do_read(&mut self) -> T {
        self.receiver.do_receive().await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn message_channel<T>() -> (MessageChannelWriter<T>, MessageChannelReader<T>)
where
    T: Debug + Send,
{
    let (sender, receiver) = mpsc::channel(16);

    (
        MessageChannelWriter::new(sender),
        MessageChannelReader::new(receiver),
    )
}
