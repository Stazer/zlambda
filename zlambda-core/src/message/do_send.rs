use tokio::sync::{oneshot, mpsc};
use std::fmt::Debug;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait DoSend<T> {
    async fn do_send(self, value: T);
}

impl<T> DoSend<T> for oneshot::Sender<T>
where
    T: Debug + Send,
{
    async fn do_send(self, value: T) {
        self.send(value).expect("Data must be sent")
    }
}

impl<T> DoSend<T> for &mpsc::Sender<T>
where
    T: Debug + Send,
{
    async fn do_send(self, value: T) {
        self.send(value).await.expect("Data must be sent")
    }
}
