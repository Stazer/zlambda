use crate::common::message::{DoReceive, DoSend};
use std::fmt::{self, Debug, Formatter};
use tokio::sync::oneshot;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn synchronized_message_output_channel() -> (
    SynchronizedMessageOutputSender,
    SynchronizedMessageOutputReceiver,
) {
    let (sender, receiver) = oneshot::channel();

    (
        SynchronizedMessageOutputSender::new(sender),
        SynchronizedMessageOutputReceiver::new(receiver),
    )
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct SynchronizedMessageOutputSender {
    sender: oneshot::Sender<()>,
}

impl SynchronizedMessageOutputSender {
    fn new(sender: oneshot::Sender<()>) -> Self {
        Self { sender }
    }

    pub async fn notify(self) {
        self.sender.do_send(()).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct SynchronizedMessageOutputReceiver {
    receiver: oneshot::Receiver<()>,
}

impl SynchronizedMessageOutputReceiver {
    fn new(receiver: oneshot::Receiver<()>) -> Self {
        Self { receiver }
    }

    pub async fn wait(self) {
        self.receiver.do_receive().await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SynchronizedMessage<I> {
    input: I,
    output_sender: SynchronizedMessageOutputSender,
}

impl<I> Debug for SynchronizedMessage<I>
where
    I: Debug,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut builder = formatter.debug_struct("SynchronizedMessage");
        builder.field("input", &self.input);
        builder.finish()
    }
}

impl<I> From<SynchronizedMessage<I>> for (I, SynchronizedMessageOutputSender) {
    fn from(envelope: SynchronizedMessage<I>) -> Self {
        (envelope.input, envelope.output_sender)
    }
}

impl<I> SynchronizedMessage<I> {
    pub fn new(input: I, output_sender: SynchronizedMessageOutputSender) -> Self {
        Self {
            input,
            output_sender,
        }
    }

    pub fn input(&self) -> &I {
        &self.input
    }

    pub fn output_sender(&self) -> &SynchronizedMessageOutputSender {
        &self.output_sender
    }
}
