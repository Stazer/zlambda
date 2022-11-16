use std::error;
use std::fmt::{self, Debug, Display, Formatter};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum ReadError<T> {
    Send(mpsc::error::SendError<T>),
    Receive(oneshot::error::RecvError),
}

impl<T> error::Error for ReadError<T> where T: Debug {}

impl<T> Debug for ReadError<T>
where
    T: Debug,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Send(error) => Debug::fmt(error, formatter),
            Self::Receive(error) => Debug::fmt(error, formatter),
        }
    }
}

impl<T> Display for ReadError<T> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Send(error) => Display::fmt(error, formatter),
            Self::Receive(error) => Display::fmt(error, formatter),
        }
    }
}

impl<T> From<mpsc::error::SendError<T>> for ReadError<T> {
    fn from(error: mpsc::error::SendError<T>) -> Self {
        Self::Send(error)
    }
}

impl<T> From<oneshot::error::RecvError> for ReadError<T> {
    fn from(error: oneshot::error::RecvError) -> Self {
        Self::Receive(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ReadMessage<T> = Box<dyn FnOnce(&T) + Send>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ReadSender<T>(mpsc::Sender<ReadMessage<T>>);

impl<T> Clone for ReadSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> ReadSender<T>
where
    T: Sync + 'static,
{
    pub async fn send<'a, F, R>(&self, reader: F) -> Result<R, ReadError<ReadMessage<T>>>
    where
        F: Fn(&T) -> R + Send + 'a + 'static,
        R: Send + Debug + 'static,
    {
        let (sender, receiver) = oneshot::channel::<R>();

        self.0
            .send(Box::new(move |x| {
                sender.send(reader(x)).expect("Cannot send value");
            }))
            .await?;

        receiver.await.map_err(ReadError::from)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ReadReceiver<T> = mpsc::Receiver<ReadMessage<T>>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn read_channel<T>() -> (ReadSender<T>, ReadReceiver<T>) {
    let (sender, receiver) = mpsc::channel(16);

    (ReadSender(sender), receiver)
}
