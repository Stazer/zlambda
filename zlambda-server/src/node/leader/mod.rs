pub mod client;
pub mod connection;
pub mod follower;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::algorithm::next_key;
use crate::log::leading::LeadingLog;
use crate::state::State;
use connection::LeaderNodeConnection;
use follower::LeaderNodeFollowerMessage;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, trace};
use zlambda_common::log::{ClusterLogEntryType, LogEntryData, LogEntryId, LogEntryType};
use zlambda_common::message::{MessageStreamReader, MessageStreamWriter};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LeaderNodeMessage {
    Acknowledge {
        log_entry_ids: Vec<LogEntryId>,
        node_id: NodeId,
    },
    Register {
        address: SocketAddr,
        follower_sender: mpsc::Sender<LeaderNodeFollowerMessage>,
        result_sender: oneshot::Sender<(NodeId, NodeId, Term, HashMap<NodeId, SocketAddr>)>,
    },
    Replicate {
        log_entry_type: LogEntryType,
        result_sender: oneshot::Sender<LogEntryId>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNode {
    id: NodeId,
    log: LeadingLog,
    term: Term,
    tcp_listener: TcpListener,
    addresses: HashMap<NodeId, SocketAddr>,
    sender: mpsc::Sender<LeaderNodeMessage>,
    receiver: mpsc::Receiver<LeaderNodeMessage>,
    state: State,
    follower_senders: HashMap<NodeId, mpsc::Sender<LeaderNodeFollowerMessage>>,
}

impl LeaderNode {
    pub fn new(tcp_listener: TcpListener) -> Result<Self, Box<dyn Error>> {
        let (sender, receiver) = mpsc::channel(16);

        let address = tcp_listener.local_addr()?;

        Ok(Self {
            id: 0,
            log: LeadingLog::default(),
            term: 0,
            tcp_listener,
            addresses: [(0, address)].into(),
            sender,
            receiver,
            state: State::default(),
            follower_senders: HashMap::default(),
        })
    }

    pub async fn run(&mut self) {
        loop {
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
                    let message = match receive_result {
                        None => {
                            error!("Nothing received");
                            break
                        }
                        Some(message) => message,
                    };

                    match message {
                        LeaderNodeMessage::Register { address, follower_sender, result_sender  } => self.register(
                            address,
                            follower_sender,
                            result_sender,
                        ).await,
                        LeaderNodeMessage::Replicate { log_entry_type, result_sender } => {
                            let id = self.replicate(log_entry_type).await;

                            if result_sender.send(id).is_err() {
                                error!("Cannot send result");
                            }
                        }
                        LeaderNodeMessage::Acknowledge { log_entry_ids, node_id } => self.acknowledge(log_entry_ids, node_id),
                    }
                }
            )
        }
    }

    async fn register(
        &mut self,
        address: SocketAddr,
        follower_sender: mpsc::Sender<LeaderNodeFollowerMessage>,
        result_sender: oneshot::Sender<(NodeId, NodeId, Term, HashMap<NodeId, SocketAddr>)>,
    ) {
        let id = next_key(self.addresses.keys());
        self.addresses.insert(id, address);

        self.replicate(LogEntryType::Cluster(ClusterLogEntryType::Addresses(
            self.addresses.clone(),
        )))
        .await;

        self.follower_senders.insert(id, follower_sender);

        trace!("Node {} registered", id);

        if result_sender
            .send((id, self.id, self.term, self.addresses.clone()))
            .is_err()
        {
            error!("Cannot send result");
        }
    }

    async fn replicate(&mut self, log_entry_type: LogEntryType) -> LogEntryId {
        let id = self.log.begin(
            log_entry_type.clone(),
            self.addresses.keys().copied().collect(),
        );

        for follower_sender in self.follower_senders.values() {
            let log_entry_data = LogEntryData::new(id, log_entry_type.clone());

            if follower_sender
                .send(LeaderNodeFollowerMessage::Replicate {
                    term: self.term,
                    log_entry_data: vec![log_entry_data],
                })
                .await
                .is_err()
            {
                error!("Cannot send LeaderNodeFollowerMessage");
            }
        }

        id
    }

    fn acknowledge(&mut self, log_entry_ids: Vec<LogEntryId>, node_id: NodeId) {
        for log_entry_id in log_entry_ids.into_iter() {
            trace!(
                "Log entry {} acknowledged by node {}",
                log_entry_id,
                node_id
            );
            self.log.acknowledge(log_entry_id, node_id);

            if let Some(log_entry) = self.log.get(log_entry_id) {
                if self.log.is_applicable(log_entry_id) {
                    match log_entry.data().r#type() {
                        LogEntryType::Client(client_log_entry_type) => {
                            self.state.apply(client_log_entry_type.clone());
                        }
                        LogEntryType::Cluster(ClusterLogEntryType::Addresses(addresses)) => {
                            self.addresses = addresses.clone();
                        }
                    }
                }
            }
        }
    }
}
