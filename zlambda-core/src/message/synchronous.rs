use std::fmt::{self, Debug, Formatter};
use tokio::sync::oneshot;
use crate::message::DoSend;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn synchronous_message_output_channel() -> () {

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
    fn new(
        sender: oneshot::Sender<O>,
    ) -> Self {
        Self {
            sender,
        }
    }

    pub async fn do_send(self, output: O) {
        self.sender.do_send(output).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SynchronousMessage<I, O> {
    input: I,
    sender: oneshot::Sender<O>,
}

impl<I, O> Debug for SynchronousMessage<I, O>
where
    I: Debug,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut builder = formatter.debug_struct("AsynchronousMessage");
        builder.field("input", &self.input);
        builder.finish()
    }
}

impl<I, O> From<SynchronousMessage<I, O>> for (I, oneshot::Sender<O>) {
    fn from(envelope: SynchronousMessage<I, O>) -> Self {
        (envelope.input, envelope.sender)
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
