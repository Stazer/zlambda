use crate::common::future::{Sink, SinkExt, Stream};
use crate::common::message::{
    synchronizable_message_output_channel, synchronous_message_output_channel, AsynchronousMessage,
    DoReceive, DoSend, SynchronizableMessage, SynchronousMessage,
};
use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tokio_util::sync::PollSender;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MessageQueueSender<T>
where
    T: Debug + Send,
{
    sender: mpsc::Sender<T>,
}

impl<T> Clone for MessageQueueSender<T>
where
    T: Debug + Send,
{
    fn clone(&self) -> Self {
        Self::new(self.sender.clone())
    }
}

/*impl<T> Sink<T> for MessageQueueSender<T>
where
    T: Debug + Send + 'static,
{
    type Error = <PollSender<T> as Sink<T>>::Error;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.sender).poll_ready(context)
    }

    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        Pin::new(&mut self.sender).start_send(item)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.sender).poll_flush(context)
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.sender).poll_close(context)
    }
}*/

impl<T> MessageQueueSender<T>
where
    T: Debug + Send,
{
    fn new(sender: mpsc::Sender<T>) -> Self {
        Self { sender }
    }

    pub async fn do_send<M>(&self, message: M)
    where
        T: From<M>,
    {
        self.sender.do_send(T::from(message)).await;
    }

    pub async fn do_send_asynchronous<I>(&self, input: I)
    where
        T: From<AsynchronousMessage<I>>,
    {
        self.do_send(AsynchronousMessage::new(input)).await
    }

    pub async fn do_send_synchronous<I, O>(&self, input: I) -> O
    where
        T: From<SynchronousMessage<I, O>>,
        O: Debug + Send,
    {
        let (sender, receiver) = synchronous_message_output_channel();

        self.do_send(SynchronousMessage::new(input, sender)).await;

        receiver.do_receive().await
    }

    pub async fn do_send_synchronized<I>(&self, input: I)
    where
        T: From<SynchronizableMessage<I>>,
    {
        let (sender, receiver) = synchronizable_message_output_channel();

        self.do_send(SynchronizableMessage::new(input, Some(sender)))
            .await;

        receiver.wait().await;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct MessageQueueReceiver<T>
where
    T: Debug + Send,
{
    receiver: mpsc::Receiver<T>,
}

impl<T> Stream for MessageQueueReceiver<T>
where
    T: Debug + Send,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(context)
    }
}

impl<T> MessageQueueReceiver<T>
where
    T: Debug + Send,
{
    fn new(receiver: mpsc::Receiver<T>) -> Self {
        Self { receiver }
    }

    pub async fn do_receive(&mut self) -> T {
        self.receiver.do_receive().await
    }

    pub async fn receive(&mut self) -> Option<T> {
        self.receiver.recv().await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn message_queue<T>() -> (MessageQueueSender<T>, MessageQueueReceiver<T>)
where
    T: Debug + Send + 'static,
{
    let (sender, receiver) = mpsc::channel(16);

    (
        MessageQueueSender::new(sender),
        MessageQueueReceiver::new(receiver),
    )
}
