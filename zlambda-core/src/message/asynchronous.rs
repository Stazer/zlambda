use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::fmt::{self, Debug, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct AsynchronousMessage<I> {
    input: I,
}

impl<I> Clone for AsynchronousMessage<I>
where
    I: Clone,
{
    fn clone(&self) -> Self {
        Self::new(self.input.clone())
    }
}

impl<I> Debug for AsynchronousMessage<I>
where
    I: Debug,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut builder = formatter.debug_struct("AsynchronousMessage");
        builder.field("input", &self.input);
        builder.finish()
    }
}

impl<'d, I> Deserialize<'d> for AsynchronousMessage<I>
where
    I: Deserialize<'d>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'d>,
    {
        Ok(Self::new(I::deserialize(deserializer)?))
    }
}

impl<I> From<AsynchronousMessage<I>> for (I,) {
    fn from(envelope: AsynchronousMessage<I>) -> Self {
        (envelope.input,)
    }
}

impl<I> Serialize for AsynchronousMessage<I>
where
    I: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("AsychronousMessage", &self.input)
    }
}

impl<I> AsynchronousMessage<I> {
    pub fn new(input: I) -> Self {
        Self { input }
    }

    pub fn input(&self) -> &I {
        &self.input
    }
}
