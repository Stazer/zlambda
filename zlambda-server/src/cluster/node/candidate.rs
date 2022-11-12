use crate::tcp::{TcpListenerMessage, TcpStreamMessage};
use bytes::Bytes;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{select, spawn};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterCandidateNodeMessage {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterCandidateNodeActorSender {
    tcp_listener_message: Sender<TcpListenerMessage>,
    tcp_stream_message: Sender<TcpStreamMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterCandidateNodeActorReceiver {
    cluster_follower_node_message: Receiver<ClusterCandidateNodeMessage>,
    tcp_stream_result: Receiver<Result<Bytes, io::Error>>,
    tcp_listener_result: Receiver<Result<(TcpStream, SocketAddr), io::Error>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterCandidateNodeActor {
    sender: ClusterCandidateNodeActorSender,
    receiver: ClusterCandidateNodeActorReceiver,
}

impl ClusterCandidateNodeActor {
    pub fn new(
        tcp_listener_message_sender: Sender<TcpListenerMessage>,
        tcp_listener_result_receiver: Receiver<Result<(TcpStream, SocketAddr), io::Error>>,
        tcp_stream_message_sender: Sender<TcpStreamMessage>,
        tcp_stream_result_receiver: Receiver<Result<Bytes, io::Error>>,
    ) -> Sender<ClusterCandidateNodeMessage> {
        let (cluster_follower_node_message_sender, cluster_follower_node_message_receiver) =
            channel(16);

        spawn(async move {
            (Self {
                sender: ClusterCandidateNodeActorSender {
                    tcp_listener_message: tcp_listener_message_sender,
                    tcp_stream_message: tcp_stream_message_sender,
                },
                receiver: ClusterCandidateNodeActorReceiver {
                    tcp_stream_result: tcp_stream_result_receiver,
                    cluster_follower_node_message: cluster_follower_node_message_receiver,
                    tcp_listener_result: tcp_listener_result_receiver,
                },
            })
            .main()
            .await;
        });

        cluster_follower_node_message_sender
    }

    async fn main(&mut self) {
        select!(
            message = self.receiver.cluster_follower_node_message.recv() => {

            },
            result = self.receiver.tcp_stream_result.recv() => {

            },
            result = self.receiver.tcp_listener_result.recv() => {

            },
        )
    }
}
