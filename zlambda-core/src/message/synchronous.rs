use crate::message::{DoReceive, DoSend};
use std::fmt::{self, Debug, Formatter};
use tokio::sync::oneshot;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn synchronous_message_output_channel<O>() -> (
    SynchronousMessageOutputSender<O>,
    SynchronousMessageOutputReceiver<O>,
)
where
    O: Debug + Send,
{
    let (sender, receiver) = oneshot::channel();

    (
        SynchronousMessageOutputSender::new(sender),
        SynchronousMessageOutputReceiver::new(receiver),
    )
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SynchronousMessageOutputSender<O>
where
    O: Debug + Send,
{
    sender: oneshot::Sender<O>,
}

impl<O> SynchronousMessageOutputSender<O>
where
    O: Debug + Send,
{
    fn new(sender: oneshot::Sender<O>) -> Self {
        Self { sender }
    }

    pub async fn do_send<T>(self, output: T)
    where
        O: From<T>,
    {
        self.sender.do_send(output.into()).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SynchronousMessageOutputReceiver<O>
where
    O: Send,
{
    receiver: oneshot::Receiver<O>,
}

impl<O> SynchronousMessageOutputReceiver<O>
where
    O: Debug + Send,
{
    fn new(receiver: oneshot::Receiver<O>) -> Self {
        Self { receiver }
    }

    pub async fn do_receive(self) -> O {
        self.receiver.do_receive().await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SynchronousMessage<I, O>
where
    O: Debug + Send,
{
    input: I,
    output_sender: SynchronousMessageOutputSender<O>,
}

impl<I, O> Debug for SynchronousMessage<I, O>
where
    I: Debug,
    O: Debug + Send,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut builder = formatter.debug_struct("SynchronousMessage");
        builder.field("input", &self.input);
        builder.finish()
    }
}

impl<I, O> From<SynchronousMessage<I, O>> for (I, SynchronousMessageOutputSender<O>)
where
    O: Debug + Send,
{
    fn from(envelope: SynchronousMessage<I, O>) -> Self {
        (envelope.input, envelope.output_sender)
    }
}

impl<I, O> SynchronousMessage<I, O>
where
    O: Debug + Send,
{
    pub fn new(input: I, output_sender: SynchronousMessageOutputSender<O>) -> Self {
        Self {
            input,
            output_sender,
        }
    }

    pub fn input(&self) -> &I {
        &self.input
    }

    pub fn output_sender(&self) -> &SynchronousMessageOutputSender<O> {
        &self.output_sender
    }
}
