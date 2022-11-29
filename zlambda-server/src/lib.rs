#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod candidate;
pub mod follower;
pub mod leader;
pub mod state;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::candidate::Candidate;
use crate::follower::Follower;
use crate::leader::Leader;
use std::error::Error;
use tokio::net::{TcpListener, ToSocketAddrs};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum NodeType {
    Leader(Leader),
    Follower(Follower),
    Candidate(Candidate),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Server {
    r#type: NodeType,
}

impl Server {
    pub async fn new<S, T>(
        listener_address: S,
        registration_address: Option<T>,
    ) -> Result<Self, Box<dyn Error>>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;

        let node = match registration_address {
            None => Self {
                r#type: NodeType::Leader(Leader::new(tcp_listener)?),
            },
            Some(registration_address) => Self {
                r#type: NodeType::Follower(
                    Follower::new(tcp_listener, registration_address).await?,
                ),
            },
        };

        Ok(node)
    }

    pub async fn run(mut self) {
        loop {
            match &mut self.r#type {
                NodeType::Leader(leader) => leader.run().await,
                NodeType::Follower(follower) => follower.run().await,
                NodeType::Candidate(candidate) => candidate.run().await,
            }
        }
    }
}
