use async_trait::async_trait;
use tokio::sync::{mpsc, oneshot};

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
