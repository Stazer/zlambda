use crate::cluster::node::leader::ClusterLeaderNodeMessage;
use crate::cluster::{ClusterMessage, ClusterMessageReader};
use crate::tcp::TcpStreamMessage;
use bytes::Bytes;
use std::io;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterLeaderNodeConnectionMessage {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterLeaderNodeConnectionActorSender {
    cluster_leader_node_message: mpsc::Sender<ClusterLeaderNodeMessage>,
    tcp_stream_message: mpsc::Sender<TcpStreamMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterLeaderNodeConnectionActorReceiver {
    tcp_stream_result: mpsc::Receiver<Result<Bytes, io::Error>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterLeaderNodeConnectionActor {
    sender: ClusterLeaderNodeConnectionActorSender,
    receiver: ClusterLeaderNodeConnectionActorReceiver,
    cluster_message_reader: ClusterMessageReader,
}

impl ClusterLeaderNodeConnectionActor {
    pub fn new(
        tcp_stream_result_receiver: mpsc::Receiver<Result<Bytes, io::Error>>,
        tcp_stream_message_sender: mpsc::Sender<TcpStreamMessage>,
        cluster_leader_node_message_sender: mpsc::Sender<ClusterLeaderNodeMessage>,
    ) {
        spawn(async move {
            (Self {
                sender: ClusterLeaderNodeConnectionActorSender {
                    tcp_stream_message: tcp_stream_message_sender,
                    cluster_leader_node_message: cluster_leader_node_message_sender,
                },
                receiver: ClusterLeaderNodeConnectionActorReceiver {
                    tcp_stream_result: tcp_stream_result_receiver,
                },
                cluster_message_reader: ClusterMessageReader::default(),
            })
            .main()
            .await;
        });
    }

    async fn main(&mut self) {
        'select: loop {
            select!(
                result = self.receiver.tcp_stream_result.recv() => {
                    let data = match result {
                        None => break,
                        Some(Err(error)) => {
                            error!("{}", error);
                            break;
                        }
                        Some(Ok(data)) => data,
                    };

                    self.cluster_message_reader.push(data);

                    'message_read: loop {
                        let message = match self.cluster_message_reader.next() {
                            Ok(None) => break 'message_read,
                            Err(error) => {
                                error!("{}", error);
                                break 'select;
                            }
                            Ok(Some(message)) => message,
                        };

                        match message {
                            ClusterMessage::RegisterRequest => {
                                let (sender, receiver) = oneshot::channel();

                                let result = self.sender.cluster_leader_node_message.send(
                                    ClusterLeaderNodeMessage::RegisterClusterNode {
                                        sender,
                                        socket_address: todo!(),
                                    },
                                ).await;

                                if let Err(error) = result {
                                    error!("{}", error);
                                    break 'select;
                                }

                                let (cluster_node_id, cluster_leader_node_id, cluster_node_addresses, term_id) = match receiver.await {
                                    Err(error) => {
                                        error!("{}", error);
                                        break 'select;
                                    },
                                    Ok(data) => data,
                                };

                                let message = ClusterMessage::RegisterResponse {
                                    cluster_node_id,
                                    cluster_leader_node_id,
                                    cluster_node_addresses,
                                    term_id,
                                };

                                let bytes = match message.to_bytes() {
                                    Ok(bytes) => bytes,
                                    Err(error) => {
                                        error!("{}", error);
                                        break 'select
                                    }
                                };

                                let result = self.sender.tcp_stream_message.send(
                                    TcpStreamMessage::SendBytes(bytes),
                                ).await;

                                if let Err(error) = result {
                                    error!("{}", error);
                                    break 'select
                                }
                            },
                            message => {
                                error!("Unhandled message {:?}", message);
                                break 'select
                            },
                        };
                    }
                }
            );
        }
    }
}
