use tokio::sync::oneshot;
use std::fmt::{self, Formatter, Debug};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AsynchronousMessageEnvelope<I> {
    input: I,
}

impl<I> Debug for AsynchronousMessageEnvelope<I>
where
    I: Debug
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut builder = formatter.debug_struct("AsynchronousMessageEnvelope");
        builder.field("input", &self.input);
        builder.finish()
    }
}

impl<I> From<AsynchronousMessageEnvelope<I>> for (I,) {
    fn from(envelope: AsynchronousMessageEnvelope<I>) -> Self {
        (envelope.input,)
    }
}

impl<I> AsynchronousMessageEnvelope<I> {
    pub fn new(input: I) -> Self {
        Self {
            input,
        }
    }

    pub fn input(&self) -> &I {
        &self.input
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct SynchronousMessageEnvelope<I, O> {
    input: I,
    sender: oneshot::Sender<O>,
}

impl<I, O> Debug for SynchronousMessageEnvelope<I, O>
where
    I: Debug,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut builder = formatter.debug_struct("AsynchronousMessageEnvelope");
        builder.field("input", &self.input);
        builder.finish()
    }
}

impl<I, O> From<SynchronousMessageEnvelope<I, O>> for (I, oneshot::Sender<O>) {
    fn from(envelope: SynchronousMessageEnvelope<I, O>) -> Self {
        (envelope.input, envelope.sender)
    }
}

impl<I, O> SynchronousMessageEnvelope<I, O> {
    pub fn new(input: I, sender: oneshot::Sender<O>) -> Self {
        Self {
            input,
            sender,
        }
    }

    pub fn input(&self) -> &I {
        &self.input
    }

    pub fn sender(&self) -> &oneshot::Sender<O> {
        &self.sender
    }
}
