use crate::common::message::{DoReceive, DoSend};
use std::fmt::{self, Debug, Formatter};
use tokio::sync::oneshot;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn synchronizable_message_output_channel() -> (
    SynchronizableMessageOutputSender,
    SynchronizableMessageOutputReceiver,
) {
    let (sender, receiver) = oneshot::channel();

    (
        SynchronizableMessageOutputSender::new(sender),
        SynchronizableMessageOutputReceiver::new(receiver),
    )
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct SynchronizableMessageOutputSender {
    sender: oneshot::Sender<()>,
}

impl SynchronizableMessageOutputSender {
    fn new(sender: oneshot::Sender<()>) -> Self {
        Self { sender }
    }

    pub async fn notify(self) {
        self.sender.do_send(()).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct SynchronizableMessageOutputReceiver {
    receiver: oneshot::Receiver<()>,
}

impl SynchronizableMessageOutputReceiver {
    fn new(receiver: oneshot::Receiver<()>) -> Self {
        Self { receiver }
    }

    pub async fn wait(self) {
        self.receiver.do_receive().await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SynchronizableMessage<I> {
    input: I,
    output_sender: Option<SynchronizableMessageOutputSender>,
}

impl<I> Debug for SynchronizableMessage<I>
where
    I: Debug,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut builder = formatter.debug_struct("SynchronizableMessage");
        builder.field("input", &self.input);
        builder.finish()
    }
}

impl<I> From<SynchronizableMessage<I>> for (I, Option<SynchronizableMessageOutputSender>) {
    fn from(envelope: SynchronizableMessage<I>) -> Self {
        (envelope.input, envelope.output_sender)
    }
}

impl<I> SynchronizableMessage<I> {
    pub fn new(input: I, output_sender: Option<SynchronizableMessageOutputSender>) -> Self {
        Self {
            input,
            output_sender,
        }
    }

    pub fn input(&self) -> &I {
        &self.input
    }

    pub fn output_sender(&self) -> &Option<SynchronizableMessageOutputSender> {
        &self.output_sender
    }
}
