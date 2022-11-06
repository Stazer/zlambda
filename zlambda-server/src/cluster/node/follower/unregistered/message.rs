use actix::{
    Message
};
use tokio::net::ToSocketAddrs;
use std::fmt::Debug;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RegisterActorMessage<T>
where
    T: ToSocketAddrs + Send + Sync + Debug + 'static,
{
    address: T,
}

impl<T> From<RegisterActorMessage<T>> for (T,)
where
    T: ToSocketAddrs + Send + Sync + Debug + 'static,
{
    fn from(message: RegisterActorMessage<T>) -> Self {
        (message.address,)
    }
}

impl<T> Message for RegisterActorMessage<T>
where
    T: ToSocketAddrs + Send + Sync + Debug + 'static,
{
    type Result = ();
}

impl<T> RegisterActorMessage<T>
where
    T: ToSocketAddrs + Send + Sync + Debug + 'static,
{
    pub fn new(address: T) -> Self {
        Self { address }
    }

    pub fn address(&self) -> &T {
        &self.address
    }
}
