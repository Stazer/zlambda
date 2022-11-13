//pub mod candidate;
//pub mod follower;
pub mod leader;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::tcp::{TcpListenerActor, TcpStreamActor};
//use candidate::ClusterCandidateNodeMessage;
//use follower::{ClusterFollowerNodeActor, ClusterFollowerNodeMessage};
use leader::{ClusterLeaderNodeActor, ClusterLeaderNodeIncomingSender};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;
use tokio::{select, spawn};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClusterNodeId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterNodeIncomingMessage {
    Update(ClusterNodeActor),
}

pub type ClusterNodeIncomingSender = mpsc::Sender<ClusterNodeIncomingMessage>;
pub type ClusterNodeIncomingReceiver = mpsc::Receiver<ClusterNodeIncomingMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn cluster_node_incoming_channel() -> (ClusterNodeIncomingSender, ClusterNodeIncomingReceiver) {
    mpsc::channel(16)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterNodeActor {
    Leader {
        cluster_leader_node_incoming_sender: ClusterLeaderNodeIncomingSender,
        cluster_node_incoming_receiver: ClusterNodeIncomingReceiver,
    },
}

impl ClusterNodeActor {
    pub async fn new<S, T>(
        listener_address: S,
        registration_address: Option<T>,
    ) -> Result<ClusterNodeIncomingSender, Box<dyn Error>>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let (cluster_node_incoming_sender, cluster_node_incoming_receiver) =
            cluster_node_incoming_channel();

        let mut instance = match registration_address {
            None => Self::Leader {
                cluster_leader_node_incoming_sender: ClusterLeaderNodeActor::new(
                    listener_address,
                    cluster_node_incoming_sender.clone(),
                )
                .await?,
                cluster_node_incoming_receiver,
            },
            Some(registration_address) => {
                todo!()
            }
        };

        spawn(async move {
            instance.main().await;
        });

        Ok(cluster_node_incoming_sender)
    }

    async fn main(&mut self) {
        loop {
            match self {
                Self::Leader {
                    cluster_node_incoming_receiver,
                    ..
                } => {
                    select!(
                        message = cluster_node_incoming_receiver.recv() => {
                            let message = match message {
                                None => {
                                    break
                                },
                                Some(message) => message,
                            };
                        }
                    )
                }
            }
            /*select!(
                message = self.cluster_node_incoming_receiver.recv() => {
                    let message = match message {
                        None => {
                            break
                        },
                        Some(message) => message,
                    };

                    match message {
                        ClusterNodeIncomingMessage::UpdateSenderType(sender_type) => {
                            self.sender_type = sender_type;
                        }
                    };
                }
            )*/
        }
    }
}
