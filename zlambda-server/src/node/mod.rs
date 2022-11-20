pub mod candidate;
pub mod follower;
pub mod leader;

////////////////////////////////////////////////////////////////////////////////////////////////////

use candidate::CandidateNode;
use follower::FollowerNode;
use leader::LeaderNode;
use std::error::Error;
use tokio::net::{TcpListener, ToSocketAddrs};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeType {
    Leader(LeaderNode),
    Follower(FollowerNode),
    Candidate(CandidateNode),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Node {
    r#type: NodeType,
}

impl Node {
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
                r#type: NodeType::Leader(LeaderNode::new(tcp_listener)?),
            },
            Some(registration_address) => Self {
                r#type: NodeType::Follower(
                    FollowerNode::new(tcp_listener, registration_address).await?,
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
