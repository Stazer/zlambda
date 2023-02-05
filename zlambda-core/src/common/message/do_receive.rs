use std::fmt::Debug;
use tokio::sync::{mpsc, oneshot};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait DoReceive<T> {
    async fn do_receive(self) -> T;
}

impl<T> DoReceive<T> for oneshot::Receiver<T>
where
    T: Debug + Send,
{
    async fn do_receive(self) -> T {
        self.await.expect("Data must be received")
    }
}

impl<T> DoReceive<T> for &mut mpsc::Receiver<T>
where
    T: Debug + Send,
{
    async fn do_receive(self) -> T {
        self.recv().await.expect("Data must be received")
    }
}
