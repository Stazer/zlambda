pub mod deserialize;
pub mod message;
pub mod module;
pub mod notification;
pub mod serialize;
mod tagged;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod utility {
    pub use super::tagged::*;
    pub use bytes::{BufMut, Bytes, BytesMut};
    pub use serde::*;
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use async_trait::async_trait;

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

/*pub mod experimental {
    use crate::common::runtime::{select, spawn};
    use std::mem::take;
    use futures::FutureExt;
    use futures::future::BoxFuture;
    use std::future::Future;
    use tokio::sync::{mpsc, oneshot};

    enum ActorMessage<T>
    where
        T: Send + Sync + 'static,
    {
        Handle {
            function: Box<dyn FnOnce(&mut T) -> BoxFuture<'static, ()> + Send + Sync>,
        },
        Wait {
            sender: oneshot::Sender<()>,
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

        pub async fn handle<'a, H, F, R>(&self, function: H) -> R
        where
            H: FnOnce(&mut T) -> F + Send + Sync + 'static,
            F: Future<Output = R> + Send + Sync + 'static,
            R: Sync + Send + 'static,
        {
            let (sender, receiver) = oneshot::channel::<R>();

            self.sender.send(ActorMessage::Handle { function: Box::new(move |data| {
                let future = function(data);

                async move {
                    sender.send(future.await);
                }.boxed()
            })}).await;

            receiver.await.expect("")
        }

        pub async fn wait(&self) {
            let (sender, receiver) = oneshot::channel::<()>();

            self.sender.send(ActorMessage::Wait { sender }).await;

            receiver.await;
        }
    }

    pub struct ActorTask<T>
    where
        T: Send + Sync + 'static,
    {
        sender: mpsc::Sender<ActorMessage<T>>,
        receiver: mpsc::Receiver<ActorMessage<T>>,
        data: T,
        running: bool,
        exit: Vec<oneshot::Sender<()>>,
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
                exit: Vec::default(),
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

            for exit in take(&mut self.exit).into_iter() {
                exit.send(());
            }
        }

        async fn select(&mut self) {
            select!(
                result = self.receiver.recv() => {
                    match result.unwrap() {
                        ActorMessage::Handle { function } => {
                            function(&mut self.data).await
                        },
                        ActorMessage::Wait { sender } => {
                            self.exit.push(sender);
                        }
                    }
                }
            )
        }
    }
}*/
