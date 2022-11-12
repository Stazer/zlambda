use crate::cluster::{ClusterMessage, ClusterMessageReader};
use crate::tcp::TcpStreamMessage;
use bytes::Bytes;
use std::io;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterLeaderNodeConnectionMessage {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterLeaderNodeConnectionActorSender {
    tcp_stream_message: Sender<TcpStreamMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterLeaderNodeConnectionActorReceiver {
    tcp_stream_result: Receiver<Result<Bytes, io::Error>>,
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
        tcp_stream_result_receiver: Receiver<Result<Bytes, io::Error>>,
        tcp_stream_message_sender: Sender<TcpStreamMessage>,
    ) {
        spawn(async move {
            (Self {
                sender: ClusterLeaderNodeConnectionActorSender {
                    tcp_stream_message: tcp_stream_message_sender,
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
                                let message = match (ClusterMessage::RegisterResponse {}).to_bytes() {
                                    Ok(message) => message,
                                    Err(error) => {
                                        error!("{}", error);
                                        break 'select
                                    }
                                };

                                let result = self.sender.tcp_stream_message.send(
                                    TcpStreamMessage::SendBytes(
                                        message
                                    ),
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
