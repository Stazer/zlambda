use crate::cluster::{ClusterMessage, ClusterMessageStreamReader, ClusterMessageStreamWriter};
use futures::StreamExt;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use crate::algorithm::next_key;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::{select, spawn};
use tokio_util::io::ReaderStream;
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClusterNodeId = u64;
pub type ClusterTermId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ClusterLeaderNodeFollower {
    node_id: ClusterNodeId,
    reader: ClusterMessageStreamReader,
    writer: ClusterMessageStreamWriter,
    sender: ClusterLeaderNodeMessageSender,
}

impl ClusterLeaderNodeFollower {
    fn new(
        node_id: ClusterNodeId,
        reader: ClusterMessageStreamReader,
        writer: ClusterMessageStreamWriter,
    sender: ClusterLeaderNodeMessageSender,
    ) -> Self {
        Self {
            node_id,
            reader,
            writer,
            sender,
        }
    }

    pub fn spawn(
        node_id: ClusterNodeId,
        reader: ClusterMessageStreamReader,
        writer: ClusterMessageStreamWriter,
    sender: ClusterLeaderNodeMessageSender,
    ) {
        spawn(async move {
            Self::new(node_id, reader, writer, sender).main().await;
        });
    }

    async fn main(mut self) {
        loop {

        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ClusterLeaderNodeConnection {
    reader: ClusterMessageStreamReader,
    writer: ClusterMessageStreamWriter,
    sender: ClusterLeaderNodeMessageSender,
    address: SocketAddr,
}

impl ClusterLeaderNodeConnection {
    pub fn new(tcp_stream: TcpStream, sender: ClusterLeaderNodeMessageSender) -> Result<Self, Box<dyn Error>> {
        let address = tcp_stream.peer_addr()?;

        let (reader, writer) = tcp_stream.into_split();

        Ok(Self {
            reader: ClusterMessageStreamReader::new(reader),
            writer: ClusterMessageStreamWriter::new(writer),
            address,
            sender,
        })
    }

    pub async fn main(mut self) {
        loop {
            select!(
                message_result = self.reader.read() => {
                    let message = match message_result {
                        Err(error) => {
                            error!("{}", error);
                            break
                        }
                        Ok(None) => continue,
                        Ok(Some(message)) => message,
                    };

                    match message {
                        ClusterMessage::RegisterRequest { address } => {
                            let (sender, receiver) = oneshot::channel();

                            let address = SocketAddr::new(
                                self.address.ip(),
                                address.port(),
                            );

                            let result = self.sender.send(ClusterLeaderNodeMessage::Register { sender, address }).await;

                            if let Err(error) = result {
                                error!("{}", error);
                                break
                            }

                            let (node_id, leader_node_id, term_id, node_addresses) = match receiver.await {
                                Err(error) => {
                                    error!("{}", error);
                                    break
                                }
                                Ok(data) => data,
                            };

                            let result = self.writer.write(&ClusterMessage::RegisterResponse {
                                node_id, leader_node_id, term_id, node_addresses,
                            }).await;

                            if let Err(error) = result {
                                error!("{}", error);
                                break
                            }

                            ClusterLeaderNodeFollower::spawn(
                                node_id,
                                self.reader,
                                self.writer,
                                self.sender,
                            );

                            break
                        }
                        message => {
                            error!("Unhandled message {:?}", message);
                            break
                        }
                    };
                }
            )
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum ClusterLeaderNodeMessage {
    Register {
        address: SocketAddr,
        sender: oneshot::Sender<(
            ClusterNodeId,
            ClusterNodeId,
            ClusterTermId,
            HashMap<ClusterNodeId, SocketAddr>,
        )>,
    },
}

pub type ClusterLeaderNodeMessageSender = mpsc::Sender<ClusterLeaderNodeMessage>;
pub type ClusterLeaderNodeMessageReceiver = mpsc::Receiver<ClusterLeaderNodeMessage>;

pub fn cluster_leader_node_message_channel() -> (
    ClusterLeaderNodeMessageSender,
    ClusterLeaderNodeMessageReceiver,
) {
    mpsc::channel(16)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterNodeType {
    Leader {
        sender: ClusterLeaderNodeMessageSender,
        receiver: ClusterLeaderNodeMessageReceiver,
    },
    Follower {
        reader: ClusterMessageStreamReader,
        writer: ClusterMessageStreamWriter,
    },
    Candidate {},
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterNode {
    node_id: ClusterNodeId,
    leader_node_id: ClusterNodeId,
    term_id: ClusterTermId,
    tcp_listener: TcpListener,
    r#type: ClusterNodeType,
    node_addresses: HashMap<ClusterNodeId, SocketAddr>,
}

impl ClusterNode {
    pub async fn new<S, T>(
        listener_address: S,
        registration_address: Option<T>,
    ) -> Result<Self, Box<dyn Error>>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;

        let this = match registration_address {
            None => Self::new_leader(tcp_listener),
            Some(registration_address) => {
                Self::new_follower(tcp_listener, registration_address).await?
            }
        };

        Ok(this)
    }

    fn new_leader(tcp_listener: TcpListener) -> Self {
        let (sender, receiver) = cluster_leader_node_message_channel();

        Self {
            node_id: 0,
            term_id: 0,
            leader_node_id: 0,
            tcp_listener,
            r#type: ClusterNodeType::Leader { sender, receiver },
            node_addresses: [].into(),
        }
    }

    async fn new_follower<T>(
        tcp_listener: TcpListener,
        registration_address: T,
    ) -> Result<Self, Box<dyn Error>>
    where
        T: ToSocketAddrs,
    {
        let address = tcp_listener.local_addr()?;

        let (reader, writer) = TcpStream::connect(registration_address).await?.into_split();

        let (mut reader, mut writer) = (
            ClusterMessageStreamReader::new(reader),
            ClusterMessageStreamWriter::new(writer),
        );

        writer.write(&ClusterMessage::RegisterRequest { address }).await?;

        let (node_id, leader_node_id, node_addresses, term_id) = match reader.read().await? {
            None => return Err("Expected message".into()),
            Some(_) => return Err("Expected request response".into()),
            Some(ClusterMessage::RegisterResponse {
                node_id,
                leader_node_id,
                node_addresses,
                term_id,
            }) => (node_id, leader_node_id, node_addresses, term_id),
        };

        Ok(Self {
            node_id,
            term_id,
            leader_node_id,
            tcp_listener,
            node_addresses,
            r#type: ClusterNodeType::Follower { reader, writer },
        })
    }

    pub async fn main(mut self) {
        loop {
            match &mut self.r#type {
                ClusterNodeType::Leader {
                    ref mut sender,
                    ref mut receiver,
                } => {
                    select!(
                        accept_result = self.tcp_listener.accept() => {
                            let (socket, address) = match accept_result {
                                Err(error) => {
                                    error!("{}", error);
                                    break;
                                },
                                Ok(data) => data,
                            };

                            let future = match ClusterLeaderNodeConnection::new(
                                socket,
                                sender.clone(),
                            ) {
                                Ok(node) => node.main(),
                                Err(error) => {
                                    error!("{}", error);
                                    continue;
                                }
                            };

                            spawn(async move {
                                trace!("Created connection {}", address);
                                future.await;
                                trace!("Destroyed connection {}", address);
                            });
                        }
                        receive_result = receiver.recv() => {
                            let message = match receive_result {
                                None => continue,
                                Some(message) => message,
                            };

                            match message {
                                ClusterLeaderNodeMessage::Register { address, sender } => {
                                    let id = next_key(self.node_addresses.keys());
                                    self.node_addresses.insert(id, address);

                                    sender.send((
                                        id,
                                        self.leader_node_id,
                                        self.term_id,
                                        self.node_addresses.clone(),
                                    ));
                                }
                            };
                        }
                    );
                }
                ClusterNodeType::Follower { .. } => {}
                ClusterNodeType::Candidate {} => {}
            }
        }
    }
}
