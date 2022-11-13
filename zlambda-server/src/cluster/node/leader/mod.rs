pub mod connection;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::algorithm::next_key;
use crate::cluster::node::ClusterNodeId;
use crate::cluster::TermId;
use crate::tcp::{TcpListenerMessage, TcpStreamActor};
use connection::{ClusterLeaderNodeConnectionActor, ClusterLeaderNodeConnectionMessage};
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::select;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterLeaderNodeMessage {
    RegisterClusterNode {
        socket_address: SocketAddr,
        sender: oneshot::Sender<(
            ClusterNodeId,
            ClusterNodeId,
            HashMap<ClusterNodeId, SocketAddr>,
            TermId,
        )>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterLeaderNodeActorSender {
    _tcp_listener_message: mpsc::Sender<TcpListenerMessage>,
    cluster_leader_node_message: mpsc::Sender<ClusterLeaderNodeMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterLeaderNodeActorReceiver {
    cluster_leader_node_message: mpsc::Receiver<ClusterLeaderNodeMessage>,
    tcp_listener_result: mpsc::Receiver<Result<(TcpStream, SocketAddr), io::Error>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterLeaderNodeActor {
    cluster_node_id: ClusterNodeId,
    term_id: TermId,
    cluster_node_addresses: HashMap<ClusterNodeId, SocketAddr>,

    sender: ClusterLeaderNodeActorSender,
    receiver: ClusterLeaderNodeActorReceiver,
}

impl ClusterLeaderNodeActor {
    pub fn new(
        tcp_listener_message_sender: mpsc::Sender<TcpListenerMessage>,
        tcp_listener_result_receiver: mpsc::Receiver<Result<(TcpStream, SocketAddr), io::Error>>,
    ) -> mpsc::Sender<ClusterLeaderNodeMessage> {
        let (cluster_leader_node_message_sender, cluster_leader_node_message_receiver) =
            mpsc::channel(16);

        let result = cluster_leader_node_message_sender.clone();

        spawn(async move {
            (Self {
                sender: ClusterLeaderNodeActorSender {
                    _tcp_listener_message: tcp_listener_message_sender,
                    cluster_leader_node_message: cluster_leader_node_message_sender.clone(),
                },
                receiver: ClusterLeaderNodeActorReceiver {
                    cluster_leader_node_message: cluster_leader_node_message_receiver,
                    tcp_listener_result: tcp_listener_result_receiver,
                },
                cluster_node_id: 0,
                term_id: 0,
                cluster_node_addresses: HashMap::default(),
            })
            .main()
            .await;
        });

        result
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
                        ClusterLeaderNodeMessage::RegisterClusterNode { sender, socket_address } => {
                            let cluster_node_id = next_key(self.cluster_node_addresses.keys());
                            self.cluster_node_addresses.insert(cluster_node_id, socket_address);

                            let result = sender.send((
                                cluster_node_id,
                                self.cluster_node_id,
                                self.cluster_node_addresses.clone(),
                                self.term_id
                            ));

                            if result.is_err() {
                                self.cluster_node_addresses.remove(&cluster_node_id);
                                error!("Receiver dropped");
                            };
                        }
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

                    let (tcp_stream_result_sender, tcp_stream_result_receiver) = mpsc::channel(16);

                    ClusterLeaderNodeConnectionActor::new(
                        tcp_stream_result_receiver,
                        TcpStreamActor::new(
                            tcp_stream_result_sender,
                            stream,
                        ),
                        self.sender.cluster_leader_node_message.clone(),
                    );
                }
            )
        }
    }
}
