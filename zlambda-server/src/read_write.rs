use std::error;
use std::fmt::{self, Debug, Display, Formatter};
use tokio::sync::mpsc;
use tokio::sync::oneshot;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum ReadWriteError<T> {
    Send(mpsc::error::SendError<T>),
    Receive(oneshot::error::RecvError),
}

impl<T> error::Error for ReadWriteError<T> where T: Debug {}

impl<T> Debug for ReadWriteError<T>
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

impl<T> Display for ReadWriteError<T> {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Send(error) => Display::fmt(error, formatter),
            Self::Receive(error) => Display::fmt(error, formatter),
        }
    }
}

impl<T> From<mpsc::error::SendError<T>> for ReadWriteError<T> {
    fn from(error: mpsc::error::SendError<T>) -> Self {
        Self::Send(error)
    }
}

impl<T> From<oneshot::error::RecvError> for ReadWriteError<T> {
    fn from(error: oneshot::error::RecvError) -> Self {
        Self::Receive(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ReadWriteMessage<T> = Box<dyn FnOnce(&mut T) + Send>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ReadWriteSender<T>(mpsc::Sender<ReadWriteMessage<T>>);

impl<T> Clone for ReadWriteSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> ReadWriteSender<T>
where
    T: Sync + 'static,
{
    pub async fn send<'a, F, R>(&self, reader: F) -> Result<R, ReadWriteError<ReadWriteMessage<T>>>
    where
        F: Fn(&mut T) -> R + Send + 'a + 'static,
        R: Send + Debug + 'static,
    {
        let (sender, receiver) = oneshot::channel::<R>();

        self.0
            .send(Box::new(move |x| {
                sender.send(reader(x)).expect("Cannot send value");
            }))
            .await?;

        receiver.await.map_err(ReadWriteError::from)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ReadWriteReceiver<T> = mpsc::Receiver<ReadWriteMessage<T>>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn read_write_channel<T>() -> (ReadWriteSender<T>, ReadWriteReceiver<T>) {
    let (sender, receiver) = mpsc::channel(16);

    (ReadWriteSender(sender), receiver)
}
