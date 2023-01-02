#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod candidate;
pub mod follower;
pub mod leader;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::candidate::CandidateHandle;
use crate::follower::{FollowerBuilder, FollowerHandle};
use crate::leader::{LeaderBuilder, LeaderHandle};
use bytes::Bytes;
use std::error::Error;
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use zlambda_common::channel::{DoReceive, DoSend};
use zlambda_common::module::ModuleId;
use zlambda_common::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ServerPingMessage {
    sender: oneshot::Sender<()>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ServerDispatchMessage {
    module_id: ModuleId,
    payload: Vec<u8>,
    node_id: Option<NodeId>,
    sender: oneshot::Sender<Result<Vec<u8>, String>>,
}

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
    Ping(ServerPingMessage),
    Dispatch(ServerDispatchMessage),
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
            .do_send(ServerMessage::Ping(ServerPingMessage { sender }))
            .await;

        receiver.do_receive().await
    }

    pub async fn dispatch(
        &self,
        module_id: ModuleId,
        payload: Vec<u8>,
        node_id: Option<NodeId>,
    ) -> Result<Vec<u8>, String> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(ServerMessage::Dispatch(ServerDispatchMessage {
                module_id,
                payload,
                node_id,
                sender,
            }))
            .await;

        receiver.do_receive().await
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
        ServerTask::new(self.sender, self.receiver, listener_address, follower_data).await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerTask {
    r#type: ServerType,
    sender: mpsc::Sender<ServerMessage>,
    receiver: mpsc::Receiver<ServerMessage>,
}

impl ServerTask {
    async fn new<S, T>(
        sender: mpsc::Sender<ServerMessage>,
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

        Ok(Self {
            r#type,
            sender,
            receiver,
        })
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
            ServerMessage::Ping(message) => self.on_ping(message).await,
            ServerMessage::Dispatch(message) => self.on_dispatch(message).await,
        }
    }

    async fn on_ping(&mut self, message: ServerPingMessage) {
        message.sender.do_send(()).await
    }

    async fn on_dispatch(&mut self, message: ServerDispatchMessage) {
        match &self.r#type {
            ServerType::Leader(leader) => {
                let leader = leader.clone();

                spawn(async move {
                    message
                        .sender
                        .do_send(
                            leader
                                .dispatch(message.module_id, message.payload, message.node_id)
                                .await,
                        )
                        .await;
                });
            }
            ServerType::Follower(follower) => {
                let follower = follower.clone();

                spawn(async move {
                    message
                        .sender
                        .do_send(
                            follower
                                .dispatch(message.module_id, message.payload, message.node_id)
                                .await,
                        )
                        .await;
                });
            }
            ServerType::Candidate(_) => {
                self.sender.do_send(ServerMessage::Dispatch(message)).await;
            }
        };
    }
}
