pub mod message;
pub mod module;
pub mod notification;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod utility {
    pub use bytes::{Bytes, BytesMut};
    pub use serde::*;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod runtime {
    pub use tokio::{join, select, spawn};
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod net {
    pub use tokio::net::*;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod sync {
    pub use tokio::sync::*;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod future {
    pub use futures::*;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod io {
    pub use tokio::io::*;
    pub use tokio_util::io::*;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod fs {
    pub use tokio::fs::*;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

/*use message::MessageQueueReceiver;
use std::fmt::Debug;
use futures::future::BoxFuture;
use futures::{FutureExt};

pub enum ActorMessage<T> {
    Handle {
        function: Box<dyn FnOnce(&mut T) -> BoxFuture<'static, ()> + Sync>,
    }
}

pub struct Actor<T> {
    receiver: tokio::sync::mpsc::Receiver<ActorMessage<T>>,
    inner: T,
    running: bool,
}

impl<T> Actor<T> {
    pub fn spawn(self) {
        spawn(async move {
            self.run().await
        })
    }

    pub async fn run(&mut self) {
        tokio::select!(
            result = self.receiver.recv() => {
                match result.unwrap() {
                    ActorMessage::Handle { function } => {
                        function(&mut self.inner).await
                    }
                }
            }
        )
    }

    pub async fn ok(sender: tokio::sync::mpsc::Sender<ActorMessage<T>>) {
        sender.send(ActorMessage::Handle { function: Box::new(move |_| {
            async move {
                ()
            }.boxed()
        })});
    }
}*/
