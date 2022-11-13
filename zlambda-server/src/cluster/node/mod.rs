pub mod candidate;
pub mod follower;
pub mod leader;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::tcp::{TcpListenerActor, TcpStreamActor};
use candidate::ClusterCandidateNodeMessage;
use follower::{ClusterFollowerNodeActor, ClusterFollowerNodeMessage};
use leader::{ClusterLeaderNodeActor, ClusterLeaderNodeMessage};
use std::error::Error;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{select, spawn};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClusterNodeId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterNodeMessage {
    UpdateType(ClusterNodeType),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterNodeType {
    Leader(Sender<ClusterLeaderNodeMessage>),
    Follower(Sender<ClusterFollowerNodeMessage>),
    Candidate(Sender<ClusterCandidateNodeMessage>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterNodeActorReceiver {
    cluster_node_message: Receiver<ClusterNodeMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterNodeActor {
    receiver: ClusterNodeActorReceiver,
    r#type: ClusterNodeType,
}

impl ClusterNodeActor {
    pub async fn new<S, T>(
        listener_address: S,
        registration_address: Option<T>,
    ) -> Result<Sender<ClusterNodeMessage>, Box<dyn Error>>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let (cluster_node_message_sender, cluster_node_message_receiver) = channel(16);

        let (tcp_listener_result_sender, tcp_listener_result_receiver) = channel(16);
        let tcp_listener_message_sender = TcpListenerActor::new(
            tcp_listener_result_sender,
            TcpListener::bind(listener_address).await?,
        );

        let r#type = match registration_address {
            None => ClusterNodeType::Leader(ClusterLeaderNodeActor::new(
                tcp_listener_message_sender,
                tcp_listener_result_receiver,
            )),
            Some(registration_address) => {
                let (tcp_stream_result_sender, tcp_stream_result_receiver) = channel(16);
                let tcp_stream_message_sender = TcpStreamActor::new(
                    tcp_stream_result_sender,
                    TcpStream::connect(registration_address).await?,
                );

                ClusterNodeType::Follower(
                    ClusterFollowerNodeActor::new(
                        tcp_listener_message_sender,
                        tcp_listener_result_receiver,
                        tcp_stream_message_sender,
                        tcp_stream_result_receiver,
                    )
                    .await?,
                )
            }
        };

        spawn(async move {
            (Self {
                receiver: ClusterNodeActorReceiver {
                    cluster_node_message: cluster_node_message_receiver,
                },
                r#type,
            })
            .main()
            .await;
        });

        Ok(cluster_node_message_sender)
    }

    async fn main(&mut self) {
        loop {
            /*select!(
                message = self.receiver.cluster_node_message.recv() => {
                    let message = match message {
                        None => {
                            break
                        },
                        Some(message) => message,
                    };

                    match message {
                        ClusterNodeMessage::UpdateType(r#type) => {
                            self.r#type = r#type;
                        }
                    };
                }
            )*/
        }
    }
}
