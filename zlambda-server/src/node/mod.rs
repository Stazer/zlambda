pub mod follower;
pub mod leader;
pub mod message;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::algorithm::next_key;
use crate::log::following::FollowingLog;
use crate::log::leading::LeadingLog;
use crate::log::{LogEntryData, LogEntryId, LogEntryType};
use crate::read_write::{read_write_channel, ReadWriteReceiver, ReadWriteSender};
use crate::state::State;
use leader::connection::LeaderNodeConnection;
use leader::follower::LeaderNodeFollower;
use message::{
    ClusterMessage, ClusterMessageRegisterResponse, Message, MessageStreamReader,
    MessageStreamWriter,
};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::{join, select, spawn};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NodeId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type Term = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeType {
    Leader {
        log: LeadingLog,
        follower_senders: HashMap<NodeId, ReadWriteSender<LeaderNodeFollower>>,
    },
    Follower {
        log: FollowingLog,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
    },
    Candidate {},
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Node {
    id: NodeId,
    leader_id: NodeId,
    term: Term,
    tcp_listener: TcpListener,
    r#type: NodeType,
    addresses: HashMap<NodeId, SocketAddr>,
    sender: ReadWriteSender<Node>,
    receiver: ReadWriteReceiver<Node>,
    state: State,
}

impl Node {
    fn new(
        id: NodeId,
        leader_id: NodeId,
        term: Term,
        tcp_listener: TcpListener,
        r#type: NodeType,
        addresses: HashMap<NodeId, SocketAddr>,
    ) -> Self {
        let (sender, receiver) = read_write_channel();

        Self {
            id,
            leader_id,
            term,
            tcp_listener,
            r#type,
            addresses,
            sender,
            receiver,
            state: State::default(),
        }
    }

    fn new_leader(tcp_listener: TcpListener) -> Self {
        Self::new(
            0,
            0,
            0,
            tcp_listener,
            NodeType::Leader {
                follower_senders: HashMap::default(),
                log: LeadingLog::default(),
            },
            [].into(),
        )
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
            MessageStreamReader::new(reader),
            MessageStreamWriter::new(writer),
        );

        writer
            .write(&Message::Cluster(ClusterMessage::RegisterRequest {
                address,
            }))
            .await?;

        let (id, leader_id, addresses, term) = match reader.read().await? {
            None => return Err("Expected message".into()),
            Some(_) => return Err("Expected request response".into()),
            Some(Message::Cluster(ClusterMessage::RegisterResponse(
                ClusterMessageRegisterResponse::Ok {
                    id,
                    leader_id,
                    addresses,
                    term,
                },
            ))) => (id, leader_id, addresses, term),
        };

        Ok(Self::new(
            id,
            leader_id,
            term,
            tcp_listener,
            NodeType::Follower {
                log: FollowingLog::default(),
                writer,
                reader,
            },
            addresses,
        ))
    }

    pub async fn spawn<S, T>(
        listener_address: S,
        registration_address: Option<T>,
    ) -> Result<(), Box<dyn Error>>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;

        let mut node = match registration_address {
            None => Self::new_leader(tcp_listener),
            Some(registration_address) => {
                Self::new_follower(tcp_listener, registration_address).await?
            }
        };

        spawn(async move {
            node.run().await;
        });

        Ok(())
    }

    async fn run(&mut self) {
        loop {
            match &mut self.r#type {
                NodeType::Leader { .. } => {
                    select!(
                        accept_result = self.tcp_listener.accept() => {
                            let (stream, address) = match accept_result {
                                Err(error) => {
                                    error!("{}", error);
                                    break
                                }
                                Ok(values) => values,
                            };

                            trace!("Connection {} created", address);

                            let (reader, writer) = stream.into_split();

                            LeaderNodeConnection::spawn(
                                MessageStreamReader::new(reader),
                                MessageStreamWriter::new(writer),
                                self.sender.clone(),
                            );
                        }
                        receive_result = self.receiver.recv() => {
                            let function = match receive_result {
                                None => {
                                    error!("Nothing received");
                                    break
                                }
                                Some(function) => function,
                            };

                            function(self)
                        }
                    )
                }
                NodeType::Follower {
                    writer,
                    reader,
                    log,
                    ..
                } => {
                    select!(
                        receive_result = self.receiver.recv() => {
                            let function = match receive_result {
                                None => {
                                    error!("Nothing received");
                                    break
                                }
                                Some(function) => function,
                            };

                            function(self)
                        }
                        read_result = reader.read() => {
                            let message = match read_result {
                                Err(error) => {
                                    error!("{}", error);
                                    break
                                }
                                Ok(None) => continue,
                                Ok(Some(message)) => message,
                            };

                            match message {
                                Message::Cluster(ClusterMessage::AppendEntriesRequest { term, log_entry_data }) => {
                                    let log_entry_ids = log_entry_data
                                        .iter()
                                        .map(|log_entry_data| log_entry_data.id())
                                        .collect();

                                    let result = writer.write(&Message::Cluster(ClusterMessage::AppendEntriesResponse {
                                        log_entry_ids
                                    })).await;

                                    if let Err(error) = result {
                                        todo!("Switch to candidate {}", error);
                                    }

                                    for log_entry_data in log_entry_data.into_iter() {
                                        log.append(log_entry_data);
                                    }
                                }
                                message => {
                                    error!("Unexpected message {:?}", message);
                                    break
                                }
                            };
                        }
                    )
                }
                NodeType::Candidate { .. } => {
                    select!(
                        receive_result = self.receiver.recv() => {
                            let function = match receive_result {
                                None => {
                                    error!("Nothing received");
                                    break
                                }
                                Some(function) => function,
                            };

                            function(self)
                        }
                    )
                }
            }
        }
    }

    fn register_follower(
        &mut self,
        address: SocketAddr,
        sender: ReadWriteSender<LeaderNodeFollower>,
    ) -> NodeId {
        let (follower_senders,) = match &mut self.r#type {
            NodeType::Leader {
                follower_senders, ..
            } => (follower_senders,),
            _ => {
                panic!("Node type must be leader");
            }
        };

        let id = next_key(self.addresses.keys());
        self.addresses.insert(id, address);
        follower_senders.insert(id, sender);

        id
    }

    pub async fn replicate(&mut self, log_entry_type: LogEntryType) -> LogEntryId {
        match &mut self.r#type {
            NodeType::Leader {
                log,
                follower_senders,
                ..
            } => {
                let id = log.begin(
                    log_entry_type.clone(),
                    self.addresses.keys().copied().collect(),
                );

                for follower_sender in follower_senders.values() {
                    let log_entry_data = LogEntryData::new(id, log_entry_type.clone());

                    /*let function = move |follower: &'_ mut LeaderNodeFollower| {
                        spawn(async move {
                            follower.replicate(log_entry_data).await
                        })
                    };*/

                    //follower_sender.send(function).await;
                }

                id
            }
            _ => {
                panic!("Node type must be leader");
            }
        }
    }

    pub fn acknowledge(&mut self, log_entry_ids: Vec<LogEntryId>, id: NodeId) {
        match &mut self.r#type {
            NodeType::Leader { log, .. } => {
                for log_entry_id in log_entry_ids.into_iter() {
                    log.acknowledge(log_entry_id, id);
                }
            }
            _ => {
                panic!("Node type must be leader");
            }
        }
    }
}
