use crate::message::{DoSend, DoReceive, AsynchronousMessage, SynchronousMessage};
use std::fmt::Debug;
use tokio::sync::{mpsc, oneshot};

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
        self.sender.do_send(T::from(message)).await
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
        let (sender, receiver) = oneshot::channel();

        self.do_send(SynchronousMessage::new(input, sender)).await;

        receiver.do_receive().await
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn message_queue<T>() -> (MessageQueueSender<T>, MessageQueueReceiver<T>)
where
    T: Debug + Send,
{
    let (sender, receiver) = mpsc::channel(16);

    (
        MessageQueueSender::new(sender),
        MessageQueueReceiver::new(receiver),
    )
}
