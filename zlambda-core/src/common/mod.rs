pub mod message;
pub mod module;
pub mod notification;
pub mod serialize;
pub mod deserialize;
mod tagged;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod utility {
    pub use super::tagged::*;
    pub use bytes::{Bytes, BytesMut, BufMut};
    pub use serde::*;
    use postcard::{take_from_bytes, serialize_with_flavor};
    use std::fmt::{self, Formatter, Display, Debug};

    pub struct ArcAny {

    }

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

pub mod bytes {
    pub use bytes::*;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod experimental {
    use crate::common::runtime::{select, spawn};
    use futures::future::BoxFuture;
    use futures::FutureExt;
    use std::future::Future;
    use tokio::sync::{mpsc, oneshot};

    enum ActorMessage<T>
    where
        T: Send + Sync + 'static,
    {
        Handle {
            function: Box<dyn FnOnce(&mut T) -> BoxFuture<'static, ()> + Send + Sync>,
        },
    }

    #[derive(Clone)]
    pub struct ActorAddress<T>
    where
        T: Send + Sync + 'static,
    {
        sender: mpsc::Sender<ActorMessage<T>>,
    }

    impl<T> ActorAddress<T>
    where
        T: Send + Sync + 'static,
    {
        fn new(sender: mpsc::Sender<ActorMessage<T>>) -> Self {
            Self { sender }
        }

        /*pub async fn handle<'a, H, R, F>(&self, function: H)
        where
            H: FnOnce(&mut T) -> F + Send + Sync + 'static,
            F: Future<Output = R> + Send + Sync,
            R: Sync + Send + 'static,
        {
            let (sender, receiver) = oneshot::channel::<R>();

            self.sender.send(ActorMessage::Handle { function: Box::new(move |data| {
                async move {
                    unsafe {
                        function(data).await;
                    }
                }.boxed()
            })});
        }*/
    }

    pub struct ActorTask<T>
    where
        T: Send + Sync + 'static,
    {
        sender: mpsc::Sender<ActorMessage<T>>,
        receiver: mpsc::Receiver<ActorMessage<T>>,
        data: T,
        running: bool,
    }

    impl<T> Default for ActorTask<T>
    where
        T: Default + Send + Sync + 'static,
    {
        fn default() -> Self {
            Self::new(T::default())
        }
    }

    impl<T> ActorTask<T>
    where
        T: Send + Sync + 'static,
    {
        pub fn new(data: T) -> Self {
            let (sender, receiver) = mpsc::channel(16);

            Self {
                sender,
                receiver,
                data,
                running: true,
            }
        }

        pub fn address(&self) -> ActorAddress<T> {
            ActorAddress::new(self.sender.clone())
        }

        pub fn spawn(mut self) {
            spawn(async move { self.run().await });
        }

        pub async fn run(&mut self) {
            while self.running {
                self.select().await
            }
        }

        async fn select(&mut self) {
            select!(
                result = self.receiver.recv() => {
                    match result.unwrap() {
                        ActorMessage::Handle { function } => {
                            function(&mut self.data).await
                        }
                    }
                }
            )
        }

        async fn ok(sender: tokio::sync::mpsc::Sender<ActorMessage<T>>) {
            sender
                .send(ActorMessage::Handle {
                    function: Box::new(move |_| async move { () }.boxed()),
                })
                .await;
        }
    }
}
