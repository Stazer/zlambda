use crate::cluster::{ClusterMessage, ClusterMessageReader};
use crate::tcp::TcpStreamMessage;
use bytes::Bytes;
use std::io;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{select, spawn};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterLeaderNodeFollowerMessage {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterLeaderNodeFollowerActorSender {
    tcp_stream_message: Sender<TcpStreamMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ClusterLeaderNodeFollowerActorReceiver {
    tcp_stream_result: Receiver<Result<Bytes, io::Error>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterLeaderNodeFollowerActor {
    sender: ClusterLeaderNodeFollowerActorSender,
    receiver: ClusterLeaderNodeFollowerActorReceiver,
    cluster_message_reader: ClusterMessageReader,
}

impl ClusterLeaderNodeFollowerActor {
    pub fn new(
        tcp_stream_result_receiver: Receiver<Result<Bytes, io::Error>>,
        tcp_stream_message_sender: Sender<TcpStreamMessage>,
        cluster_message_reader: ClusterMessageReader,
    ) {
        spawn(async move {
            (Self {
                sender: ClusterLeaderNodeFollowerActorSender {
                    tcp_stream_message: tcp_stream_message_sender,
                },
                receiver: ClusterLeaderNodeFollowerActorReceiver {
                    tcp_stream_result: tcp_stream_result_receiver,
                },
                cluster_message_reader,
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
