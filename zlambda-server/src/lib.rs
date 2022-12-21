#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod candidate;
pub mod follower;
pub mod leader;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::candidate::CandidateHandle;
use crate::follower::{FollowerBuilder, FollowerHandle};
use crate::leader::{LeaderBuilder, LeaderHandle};
use std::error::Error;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum ServerType {
    Leader(LeaderHandle),
    Follower(FollowerHandle),
    Candidate(CandidateHandle),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum ServerMessage {
    Ping { sender: oneshot::Sender<()> },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerHandle {
    sender: mpsc::Sender<ServerMessage>,
}

impl ServerHandle {
    fn new(sender: mpsc::Sender<ServerMessage>) -> Self {
        Self { sender }
    }

    pub async fn ping(&self) {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .send(ServerMessage::Ping { sender })
            .await
            .expect("");

        receiver.await.expect("");
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerBuilder {
    sender: mpsc::Sender<ServerMessage>,
    receiver: mpsc::Receiver<ServerMessage>,
}

impl ServerBuilder {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(16);

        Self { sender, receiver }
    }

    pub fn handle(&self) -> ServerHandle {
        ServerHandle::new(self.sender.clone())
    }

    pub async fn task<S, T>(
        self,
        listener_address: S,
        follower_data: Option<(T, Option<NodeId>)>,
    ) -> Result<ServerTask, Box<dyn Error>>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        ServerTask::new(self.receiver, listener_address, follower_data).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerTask {
    r#type: ServerType,
    receiver: mpsc::Receiver<ServerMessage>,
}

impl ServerTask {
    async fn new<S, T>(
        receiver: mpsc::Receiver<ServerMessage>,
        listener_address: S,
        follower_data: Option<(T, Option<NodeId>)>,
    ) -> Result<Self, Box<dyn Error>>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;

        let r#type = match follower_data {
            None => {
                let builder = LeaderBuilder::new();
                let handle = builder.handle();

                builder.task(tcp_listener)?.spawn();

                ServerType::Leader(handle)
            }
            Some((registration_address, node_id)) => {
                let builder = FollowerBuilder::new();
                let handle = builder.handle();

                builder
                    .task(tcp_listener, registration_address, node_id)
                    .await?;

                ServerType::Follower(handle)
            }
        };

        Ok(Self { r#type, receiver })
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
    }

    pub async fn run(mut self) {
        loop {
            self.select().await;
        }
    }

    async fn select(&mut self) {
        select!(
            result = self.receiver.recv() => {
                if let Some(message) = result {
                    self.on_message(message).await;
                }
            }
        )
    }

    async fn on_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Ping { sender } => self.on_ping(sender).await,
        }
    }

    async fn on_ping(&mut self, sender: oneshot::Sender<()>) {
        sender.send(()).expect("");
    }
}
