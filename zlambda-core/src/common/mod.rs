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
    use futures::future::BoxFuture;
    use futures::FutureExt;
    use std::future::Future;
    use std::mem::take;
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

        pub async fn handle<'a, H, F, R>(&'a self, function: H) -> R
        where
            H: FnOnce(&'a mut T) -> F + Send + Sync + 'a,
            F: Future<Output = R> + Send + Sync,
            R: Sync + Send + 'static,
        {
            //let func = &function as *const dyn FnOnce(&'a mut T) -> F + Send + Sync;

            let (sender, receiver) = oneshot::channel::<R>();

            self.sender
                .send(ActorMessage::Handle {
                    function: Box::new(move |data| {
                        //let future = function(data);
                        //let future = unsafe { *func  }(data);

                        async move {
                            //sender.send(future.await);
                        }
                        .boxed()
                    }),
                })
                .await;

            receiver.await.expect("")
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
                        },
                    }
                }
            )
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[tokio::test]
        async fn test1() {
            let actor = ActorTask::new(32);
            let address = actor.address();
            actor.spawn();

            address
                .handle(async move |data| {
                    println!("{}", *data);
                })
                .await;
        }
    }
}*/
