pub mod connection;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::tcp::{TcpListenerMessage, TcpStreamActor};
use crate::cluster::node::ClusterNodeId;
use connection::{ClusterLeaderNodeConnectionActor, ClusterLeaderNodeConnectionMessage};
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::select;
use tokio::spawn;
use std::collections::HashSet;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterLeaderNodeMessage {
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterLeaderNodeActorSender {
    _tcp_listener_message: Sender<TcpListenerMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterLeaderNodeActorReceiver {
    cluster_leader_node_message: Receiver<ClusterLeaderNodeMessage>,
    tcp_listener_result: Receiver<Result<(TcpStream, SocketAddr), io::Error>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterLeaderNodeActor {
    id: ClusterNodeId,
    node_ids: HashSet<ClusterNodeId>,

    sender: ClusterLeaderNodeActorSender,
    receiver: ClusterLeaderNodeActorReceiver,
}

impl ClusterLeaderNodeActor {
    pub fn new(
        tcp_listener_message_sender: Sender<TcpListenerMessage>,
        tcp_listener_result_receiver: Receiver<Result<(TcpStream, SocketAddr), io::Error>>,
    ) -> Sender<ClusterLeaderNodeMessage> {
        let (cluster_leader_node_message_sender, cluster_leader_node_message_receiver) =
            channel(16);

        spawn(async move {
            (Self {
                sender: ClusterLeaderNodeActorSender {
                    _tcp_listener_message: tcp_listener_message_sender,
                },
                receiver: ClusterLeaderNodeActorReceiver {
                    cluster_leader_node_message: cluster_leader_node_message_receiver,
                    tcp_listener_result: tcp_listener_result_receiver,
                },
                id: 0,
                node_ids: [0].into(),
            })
            .main()
            .await;
        });

        cluster_leader_node_message_sender
    }

    async fn main(&mut self) {
        loop {
            select!(
                message = self.receiver.cluster_leader_node_message.recv() => {
                    let message = match message {
                        None => break,
                        Some(message) => message,
                    };

                    match message {
                    };
                },
                result = self.receiver.tcp_listener_result.recv() => {
                    let (stream, address) = match result {
                        None => break,
                        Some(Err(error)) => {
                            error!("{}", error);
                            break;
                        }
                        Some(Ok(data)) => data,
                    };

                    trace!("Connection from {}", address);

                    let (tcp_stream_result_sender, tcp_stream_result_receiver) = channel(16);

                    ClusterLeaderNodeConnectionActor::new(
                        tcp_stream_result_receiver,
                        TcpStreamActor::new(
                            tcp_stream_result_sender,
                            stream,
                        ),
                    );
                }
            )
        }
    }
}
