mod candidate;
mod follower;
mod leader;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use candidate::*;
pub use follower::*;
pub use leader::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use actix::{Actor, Addr, Context};
use tokio::net::{ToSocketAddrs, TcpListener};
use std::net::SocketAddr;
use std::fmt::{Display, Debug};
use std::io;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum NodeActor2 {
    Leader(Addr<LeaderNodeActor>),
    Follower(Addr<FollowerNodeActor>),
    //Candidate(Addr<CandidateNodeActor>),
}

impl Actor for NodeActor2 {
    type Context = Context<Self>;
}

impl NodeActor2 {
    pub async fn new<S, T>(
        listener_address: S,
        leader_stream_address: Option<T>,
    ) -> Result<Addr<Self>, io::Error>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs + Debug + Display + Send + Sync + 'static,
    {
        let listener = TcpListener::bind(listener_address).await?;

        Ok(Self::create(move |context| {
            match leader_stream_address {
                Some(leader_stream_address) => {
                    Self::Follower(FollowerNodeActor::new(leader_stream_address))
                }
                None => {
                    Self::Leader(LeaderNodeActor::new(listener))
                }
            }
        }))
    }
}
