pub mod client;
pub mod connection;
pub mod follower;
pub mod log;

////////////////////////////////////////////////////////////////////////////////////////////////////

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
use zlambda_common::algorithm::next_key;
use zlambda_common::log::{
    ClientLogEntryType, ClusterLogEntryType, LogEntryData, LogEntryId, LogEntryType,
};
use zlambda_common::message::{MessageStreamReader, MessageStreamWriter};
use zlambda_common::module::{ModuleId, ModuleManager};
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
    Append {
        id: ModuleId,
        chunk: Vec<u8>,
        sender: Option<oneshot::Sender<()>>,
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
    module_manager: ModuleManager,
    follower_senders: HashMap<NodeId, mpsc::Sender<LeaderFollowerMessage>>,
    on_apply_initialize_module_handler:
        HashMap<LogEntryId, Box<dyn FnOnce(ModuleId) -> BoxFuture<'static, ()>>>,
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
            module_manager: ModuleManager::default(),
            follower_senders: HashMap::default(),
            on_apply_initialize_module_handler: HashMap::default(),
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
                        LeaderMessage::Append { id, chunk, sender } => self.append(id, chunk, sender).await,
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

            let (sender, receiver) = oneshot::channel();

            if let Err(error) = follower_sender
                .send(LeaderFollowerMessage::Replicate {
                    term: self.term,
                    log_entry_data: vec![log_entry_data],
                    sender,
                })
                .await
            {
                error!("Cannot send LeaderFollowerMessage::Replicate {}", error);
                continue;
            }

            if let Err(error) = receiver.await {
                error!("Cannot receive LeaderFollowerMessage::Replicate {}", error);
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

            for committed_log_entry_id in self.log.acknowledge(log_entry_id, node_id) {
                if let Some(log_entry) = self.log.get(committed_log_entry_id) {
                    match log_entry.data().r#type() {
                        LogEntryType::Client(client_log_entry_type) => {
                            self.apply(committed_log_entry_id, client_log_entry_type.clone())
                                .await;
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

        self.on_apply_initialize_module_handler.insert(
            id,
            Box::new(|module_id| {
                {
                    async move {
                        if let Err(error) = result_sender.send(module_id) {
                            error!("{}", error);
                        }
                    }
                }
                .boxed()
            }),
        );
    }

    async fn append(&mut self, id: ModuleId, chunk: Vec<u8>, sender: Option<oneshot::Sender<()>>) {
        let id = self
            .replicate(LogEntryType::Client(ClientLogEntryType::AppendModule(
                id, chunk,
            )))
            .await;

        if let Some(sender) = sender {
            if sender.send(()).is_err() {
                error!("Error sending append module");
            }
        }
    }

    async fn apply(&mut self, log_entry_id: LogEntryId, client_log_entry_type: ClientLogEntryType) {
        trace!("Apply {}", log_entry_id);

        match client_log_entry_type {
            ClientLogEntryType::InitializeModule => {
                let module_id = match self.module_manager.initialize() {
                    Err(error) => {
                        error!("{}", error);
                        return;
                    }
                    Ok(module_id) => module_id,
                };

                if let Some(handler) = self
                    .on_apply_initialize_module_handler
                    .remove(&log_entry_id)
                {
                    handler(module_id).await;
                }
            }
            ClientLogEntryType::AppendModule(id, chunk) => {
                if let Err(error) = self.module_manager.append(id, &chunk).await {
                    error!("{}", error);
                }
            }
            r#type => {
                error!("Unhandled type {:?}", r#type);
            }
        };
    }
}
