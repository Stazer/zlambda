pub mod leader;
pub mod message;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::read_write::{read_write_channel, ReadWriteReceiver, ReadWriteSender};
use leader::connection::LeaderNodeConnection;
use crate::algorithm::next_key;
use leader::LeaderNode;
use message::{
    ClusterMessage, ClusterMessageRegisterResponse, Message, MessageStreamReader,
    MessageStreamWriter,
};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::{select, spawn};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NodeId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type Term = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeType {
    Leader(LeaderNode),
    Follower {
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
        }
    }

    fn new_leader(tcp_listener: TcpListener) -> Self {
        Self::new(
            0,
            0,
            0,
            tcp_listener,
            NodeType::Leader(LeaderNode::new()),
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
            NodeType::Follower { reader, writer },
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
                NodeType::Leader(leader) => {
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
                                (*leader.sender()).clone(),
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

                        receive_result = leader.receiver_mut().recv() => {
                            let function = match receive_result {
                                None => {
                                    error!("Nothing received");
                                    break
                                }
                                Some(function) => function,
                            };

                            function(leader)
                        }
                    )
                }
                NodeType::Follower { reader, .. } => {
                    select!(
                        read_result = reader.read() => {

                        }
                    )
                }
                NodeType::Candidate { .. } => {}
            }
        }
    }

    fn register_follower(&mut self, address: SocketAddr) -> NodeId {
        assert!(matches!(self.r#type, NodeType::Leader(_)));

        let id = next_key(self.addresses.keys());
        self.addresses.insert(id, address);

        id
    }
}
