////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common::runtime::{select, spawn};
use futures::future::BoxFuture;
use futures::FutureExt;
use std::future::Future;
use std::mem::*;
use tokio::sync::{mpsc, oneshot};
use std::pin::{pin, Pin};
use std::sync::atomic::AtomicPtr;
use std::ptr::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

/*#[async_trait::async_trait]
pub trait ActorAsynchronousHandler<T>: Send + Sync + 'static
where
    T: Send + Sync + 'static,
{
    async fn handle(self: Box<Self>, instance: &mut T);
}

#[async_trait::async_trait]
impl<T, X> ActorAsynchronousHandler<T> for X
where
    T: Send + Sync + 'static,
    X: FnOnce(&mut T) -> BoxFuture<'static, ()> + Send + Sync + 'static,
{
    async fn handle(self: Box<Self>, instance: &mut T) {
        //self(instance).await;
    }
}*/

////////////////////////////////////////////////////////////////////////////////////////////////////

enum ActorMessage<T>
where
    T: Send + Sync + 'static,
{
    //Synchronous(Box<dyn FnOnce(&mut T) + Send + Sync>),
    //Asynchronous(Box<dyn FnOnce(&mut T) -> BoxFuture<'static, ()> + Send + Sync>),
    //Future(BoxFuture<'static, ()>),
    SynchronousTransition(Box<dyn FnOnce(T) -> T + Send + Sync>),
    AsynchronousTransition(Box<dyn FnOnce(T) -> BoxFuture<'static, T> + Send + Sync>),
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

    pub async fn do_synchronous_transition<H>(&self, function: H)
    where
        H: FnOnce(T) -> T + Send + Sync + 'static,
    {
        self.sender
            .send(ActorMessage::SynchronousTransition (
                Box::new(move |data| {
                    function(data)
                }),
            ))
            .await;
    }

    pub async fn do_asynchronous_transition<H, F>(&self, function: H)
    where
        H: FnOnce(T) -> F + Send + Sync + 'static,
        F: Future<Output = T> + Send,
    {
        self.sender
          .send(ActorMessage::AsynchronousTransition(
              Box::new(move |data| {
                  async move {
                      function(data).await
                  }.boxed()
                }),
            ))
            .await;
    }

    pub async fn do_asynchronous<'a, 'b, H, F>(&'b self, mut function: H)
    where
        H: FnOnce(&'a mut T) -> F + Send + Sync + 'static,
        F: Future<Output = ()> + Send + 'a + 'b + 'static,
    {
        /*let pointer = unsafe {
            std::mem::transmute::<_, *const fn(&mut T) -> F + Send>(&function)
        };*/

        //let mut function = AtomicPtr::new(addr_of_mut!(function));
        /*unsafe {
            std::mem::transmute::<_, *mut fn(&mut T) -> F>(&function)
        });*/

        self.sender
          .send(ActorMessage::AsynchronousTransition(
              Box::new(move |mut data| {
                  async move {
                      /*let inner = unsafe {
                          function.get_mut().read()
                      };*/

                      //inner(&mut data).await;

                      /*let future = unsafe {
                          (*(function.into_inner()))(&mut data)
                      };

                      future.await;*/

                      data
                  }.boxed()
                }),
            ))
            .await;
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

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        while self.running {
            self = self.select().await;
        }
    }

    async fn select(mut self) -> Self {
        select!(
            result = self.receiver.recv() => {
                match result.unwrap() {
                    ActorMessage::SynchronousTransition(transition) => {
                        self.data = transition(self.data);
                    }
                    ActorMessage::AsynchronousTransition(transition) => {
                        self.data = transition(self.data).await;
                    }
                }
            }
        );

        self
    }
}

async fn hello() {
    let actor = ActorTask::new(32);
    let address = actor.address();
    actor.spawn();

    address.do_asynchronous(async move |x| {
        println!("########################################## {}", &x);
    }).await;

    address.do_asynchronous(async move |x| {
        *x = 1337;
    }).await;

    address.do_asynchronous(async move |x| {
        println!("########################################## {}", &x);
    }).await;

    futures::future::pending().await
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test1() {
        hello().await
    }
}
