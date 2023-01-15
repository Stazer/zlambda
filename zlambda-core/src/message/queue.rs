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
        self.send(value).expect("Data must be sent")
    }
}

impl<T> DoSend<T> for &mpsc::Sender<T>
where
    T: std::fmt::Debug + Send,
{
    async fn do_send(self, value: T) {
        self.send(value).await.expect("Data must be sent")
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

#[derive(Clone, Debug)]
pub struct MessageQueueSender<T>
where
    T: Debug + Send,
{
    sender: mpsc::Sender<T>,
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
        T: From<M>
    {
        self.sender.do_send(T::from(message)).await
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
