use crate::cluster::node::ClusterNodeId;
use crate::cluster::{ClusterMessage, ClusterMessageReader, TermId};
use crate::tcp::{TcpListenerMessage, TcpStreamMessage};
use bytes::Bytes;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{select, spawn};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterFollowerNodeMessage {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterFollowerNodeActorReceiver {
    cluster_follower_node_message: Receiver<ClusterFollowerNodeMessage>,
    tcp_listener_result: Receiver<Result<(TcpStream, SocketAddr), io::Error>>,
    tcp_stream_result: Receiver<Result<Bytes, io::Error>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterFollowerNodeActorSender {
    tcp_listener_message: Sender<TcpListenerMessage>,
    tcp_stream_message: Sender<TcpStreamMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterFollowerNodeActor {
    cluster_node_id: ClusterNodeId,
    term_id: TermId,

    receiver: ClusterFollowerNodeActorReceiver,
    sender: ClusterFollowerNodeActorSender,

    cluster_message_reader: ClusterMessageReader,
}

impl ClusterFollowerNodeActor {
    pub async fn new(
        tcp_listener_message_sender: Sender<TcpListenerMessage>,
        tcp_listener_result_receiver: Receiver<Result<(TcpStream, SocketAddr), io::Error>>,
        tcp_stream_message_sender: Sender<TcpStreamMessage>,
        mut tcp_stream_result_receiver: Receiver<Result<Bytes, io::Error>>,
    ) -> Result<Sender<ClusterFollowerNodeMessage>, Box<dyn Error>> {
        let mut cluster_message_reader = ClusterMessageReader::default();

        let bytes = match (ClusterMessage::RegisterRequest {}).to_bytes() {
            Err(error) => return Err(Box::new(error)),
            Ok(bytes) => bytes,
        };

        let result = tcp_stream_message_sender
            .send(TcpStreamMessage::SendBytes(bytes))
            .await;

        if let Err(error) = result {
            return Err(Box::new(error));
        }

        let (cluster_node_id, term_id) = loop {
            let bytes = match tcp_stream_result_receiver.recv().await {
                None => return Err("Connection lost".into()),
                Some(Err(error)) => return Err(Box::new(error)),
                Some(Ok(bytes)) => bytes,
            };

            cluster_message_reader.push(bytes);

            let message = match cluster_message_reader.next() {
                Err(error) => return Err(Box::new(error)),
                Ok(None) => continue,
                Ok(Some(message)) => message,
            };

            match message {
                ClusterMessage::RegisterResponse {
                    cluster_node_id,
                    term_id,
                    cluster_leader_node_id,
                    cluster_node_addresses,
                } => break (cluster_node_id, term_id),
                message => return Err(format!("Unhandled message {:?}", message).into()),
            }
        };

        let (cluster_follower_node_message_sender, cluster_follower_node_message_receiver) =
            channel(16);

        spawn(async move {
            (Self {
                cluster_node_id,
                term_id,

                sender: ClusterFollowerNodeActorSender {
                    tcp_listener_message: tcp_listener_message_sender,
                    tcp_stream_message: tcp_stream_message_sender,
                },
                receiver: ClusterFollowerNodeActorReceiver {
                    tcp_stream_result: tcp_stream_result_receiver,
                    cluster_follower_node_message: cluster_follower_node_message_receiver,
                    tcp_listener_result: tcp_listener_result_receiver,
                },

                cluster_message_reader,
            })
            .main()
            .await;
        });

        Ok(cluster_follower_node_message_sender)
    }

    async fn main(&mut self) {
        loop {
            select!(
                message = self.receiver.cluster_follower_node_message.recv() => {
                    let message = match message {
                        None => break,
                        Some(message) => message,
                    };

                    match message {
                    };
                }
                result = self.receiver.tcp_listener_result.recv() => {
                }
                result = self.receiver.tcp_stream_result.recv() => {
                }
            )
        }
    }
}
