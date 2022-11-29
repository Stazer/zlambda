pub mod client;
pub mod connection;
pub mod follower;
pub mod log;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::algorithm::next_key;
use crate::state::State;
use connection::LeaderConnection;
use follower::LeaderFollowerMessage;
use futures::future::BoxFuture;
use futures::FutureExt;
use log::LeaderLog;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, trace};
use zlambda_common::log::{
    ClientLogEntryType, ClusterLogEntryType, LogEntryData, LogEntryId, LogEntryType,
};
use zlambda_common::message::{MessageStreamReader, MessageStreamWriter};
use zlambda_common::module::ModuleId;
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LeaderMessage {
    Acknowledge {
        log_entry_ids: Vec<LogEntryId>,
        node_id: NodeId,
    },
    Register {
        address: SocketAddr,
        follower_sender: mpsc::Sender<LeaderFollowerMessage>,
        result_sender: oneshot::Sender<(NodeId, NodeId, Term, HashMap<NodeId, SocketAddr>)>,
    },
    Replicate {
        log_entry_type: LogEntryType,
        result_sender: oneshot::Sender<LogEntryId>,
    },
    InitializeModule {
        result_sender: oneshot::Sender<ModuleId>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Leader {
    id: NodeId,
    log: LeaderLog,
    term: Term,
    tcp_listener: TcpListener,
    addresses: HashMap<NodeId, SocketAddr>,
    sender: mpsc::Sender<LeaderMessage>,
    receiver: mpsc::Receiver<LeaderMessage>,
    state: State,
    follower_senders: HashMap<NodeId, mpsc::Sender<LeaderFollowerMessage>>,
    on_apply_handler: HashMap<LogEntryId, Box<dyn FnOnce() -> BoxFuture<'static, ()>>>,
}

impl Leader {
    pub fn new(tcp_listener: TcpListener) -> Result<Self, Box<dyn Error>> {
        let (sender, receiver) = mpsc::channel(16);

        let address = tcp_listener.local_addr()?;

        Ok(Self {
            id: 0,
            log: LeaderLog::default(),
            term: 0,
            tcp_listener,
            addresses: [(0, address)].into(),
            sender,
            receiver,
            state: State::default(),
            follower_senders: HashMap::default(),
            on_apply_handler: HashMap::default(),
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

                    LeaderConnection::spawn(
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
                        LeaderMessage::Register { address, follower_sender, result_sender  } => self.register(
                            address,
                            follower_sender,
                            result_sender,
                        ).await,
                        LeaderMessage::Replicate { log_entry_type, result_sender } => {
                            let id = self.replicate(log_entry_type).await;

                            if result_sender.send(id).is_err() {
                                error!("Cannot send result");
                            }
                        }
                        LeaderMessage::Acknowledge { log_entry_ids, node_id } => self.acknowledge(log_entry_ids, node_id).await,
                        LeaderMessage::InitializeModule { result_sender,}  => self.initialize_module(result_sender).await,
                    }
                }
            )
        }
    }

    async fn register(
        &mut self,
        address: SocketAddr,
        follower_sender: mpsc::Sender<LeaderFollowerMessage>,
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
                .send(LeaderFollowerMessage::Replicate {
                    term: self.term,
                    log_entry_data: vec![log_entry_data],
                })
                .await
                .is_err()
            {
                error!("Cannot send LeaderFollowerMessage");
            }
        }

        self.sender
            .send(LeaderMessage::Acknowledge {
                log_entry_ids: vec![id],
                node_id: self.id,
            })
            .await;

        id
    }

    async fn acknowledge(&mut self, log_entry_ids: Vec<LogEntryId>, node_id: NodeId) {
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

                            if let Some(handler) = self.on_apply_handler.remove(&log_entry_id) {
                                handler().await;
                            }
                        }
                        LogEntryType::Cluster(ClusterLogEntryType::Addresses(addresses)) => {
                            self.addresses = addresses.clone();
                        }
                    }
                }
            }
        }
    }

    async fn initialize_module(&mut self, result_sender: oneshot::Sender<ModuleId>) {
        let id = self
            .replicate(LogEntryType::Client(ClientLogEntryType::InitializeModule))
            .await;

        self.on_apply_handler
            .insert(id, Box::new(|| { async move {} }.boxed()));
    }

    async fn append_module(&mut self, id: ModuleId, chunk: Vec<u8>) {}
}
