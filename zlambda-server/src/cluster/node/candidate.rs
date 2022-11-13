use bytes::Bytes;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::{select, spawn};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterCandidateNodeIncomingMessage {}

pub type ClusterCandidateNodeIncomingSender = mpsc::Sender<ClusterCandidateNodeIncomingMessage>;
pub type ClusterCandidateNodeIncomingReceiver = mpsc::Receiver<ClusterCandidateNodeIncomingMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn cluster_candidate_node_incoming_channel() -> (ClusterCandidateNodeIncomingSender, ClusterCandidateNodeIncomingReceiver) {
    mpsc::channel(16)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterCandidateNodeActor {
}

impl ClusterCandidateNodeActor {
}
