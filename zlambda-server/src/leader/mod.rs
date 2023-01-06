pub mod client;
pub mod connection;
pub mod follower;
pub mod log;

////////////////////////////////////////////////////////////////////////////////////////////////////

use bytes::Bytes;
use connection::LeaderConnectionBuilder;
use follower::LeaderFollowerHandle;
use log::LeaderLog;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::{debug, error, info, trace};
use zlambda_common::algorithm::next_key;
use zlambda_common::channel::{DoReceive, DoSend};
use zlambda_common::log::{
    ClientLogEntryType, ClusterLogEntryType, LogEntryData, LogEntryId, LogEntryType,
};
use zlambda_common::message::{MessageStreamReader, MessageStreamWriter};
use zlambda_common::module::{DispatchModuleEventInput, ModuleId, ModuleManager};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderReplicationStatus {
    term: Term,
    last_committed_log_entry_id: Option<LogEntryId>,
}

impl LeaderReplicationStatus {
    pub fn new(term: Term, last_committed_log_entry_id: Option<LogEntryId>) -> Self {
        Self {
            term,
            last_committed_log_entry_id,
        }
    }

    pub fn term(&self) -> Term {
        self.term
    }

    pub fn last_committed_log_entry_id(&self) -> &Option<LogEntryId> {
        &self.last_committed_log_entry_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum LeaderResult {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderPingMessage {
    sender: oneshot::Sender<()>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderDispatchMessage {
    module_id: ModuleId,
    payload: Vec<u8>,
    _node_id: Option<NodeId>,
    sender: oneshot::Sender<Result<Vec<u8>, String>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderAppendMessage {
    module_id: ModuleId,
    bytes: Bytes,
    sender: oneshot::Sender<()>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderReplicationStatusMessage {
    sender: oneshot::Sender<LeaderReplicationStatus>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct LeaderAcknowledgeMessage {
    node_id: NodeId,
    acknowledged_log_entry_ids: Vec<LogEntryId>,
    missing_log_entry_ids: Vec<LogEntryId>,
    sender: Option<oneshot::Sender<Vec<LogEntryData>>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum LeaderMessage {
    Ping(LeaderPingMessage),
    Register(
        SocketAddr,
        LeaderFollowerHandle,
        oneshot::Sender<(NodeId, NodeId, Term, HashMap<NodeId, SocketAddr>)>,
    ),
    Acknowledge(LeaderAcknowledgeMessage),
    Replicate(LogEntryType, oneshot::Sender<LogEntryId>),
    Initialize(oneshot::Sender<ModuleId>),
    Append(LeaderAppendMessage),
    Load(ModuleId, oneshot::Sender<Result<ModuleId, String>>),
    Dispatch(LeaderDispatchMessage),
    Handshake {
        node_id: NodeId,
        address: SocketAddr,
        sender: oneshot::Sender<Result<(Term, Option<LogEntryId>, LeaderFollowerHandle), String>>,
    },
    ApplyInitialize {
        log_entry_id: LogEntryId,
        sender: oneshot::Sender<ModuleId>,
    },
    ApplyAppend {
        log_entry_id: LogEntryId,
        sender: oneshot::Sender<()>,
    },
    ApplyLoad {
        log_entry_id: LogEntryId,
        sender: oneshot::Sender<Result<ModuleId, String>>,
    },
    ReplicationStatus(LeaderReplicationStatusMessage),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct LeaderHandle {
    sender: mpsc::Sender<LeaderMessage>,
}

impl LeaderHandle {
    fn new(sender: mpsc::Sender<LeaderMessage>) -> Self {
        Self { sender }
    }

    pub async fn ping(&self) {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderMessage::Ping(LeaderPingMessage { sender }))
            .await;

        receiver.do_receive().await
    }

    pub async fn register(
        &self,
        address: SocketAddr,
        handle: LeaderFollowerHandle,
    ) -> (NodeId, NodeId, Term, HashMap<NodeId, SocketAddr>) {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderMessage::Register(address, handle, sender))
            .await;

        receiver.do_receive().await
    }

    pub async fn handshake(
        &self,
        node_id: NodeId,
        address: SocketAddr,
    ) -> Result<(Term, Option<LogEntryId>, LeaderFollowerHandle), String> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderMessage::Handshake {
                node_id,
                address,
                sender,
            })
            .await;

        receiver.do_receive().await
    }

    pub async fn acknowledge(
        &self,
        node_id: NodeId,
        acknowledged_log_entry_ids: Vec<LogEntryId>,
        missing_log_entry_ids: Vec<LogEntryId>,
    ) -> Vec<LogEntryData> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderMessage::Acknowledge(LeaderAcknowledgeMessage {
                node_id,
                acknowledged_log_entry_ids,
                missing_log_entry_ids,
                sender: Some(sender),
            }))
            .await;

        receiver.do_receive().await
    }

    pub async fn replication_status(&self) -> LeaderReplicationStatus {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderMessage::ReplicationStatus(
                LeaderReplicationStatusMessage { sender },
            ))
            .await;

        receiver.do_receive().await
    }

    pub async fn replicate(&self, log_entry_type: LogEntryType) -> LogEntryId {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderMessage::Replicate(log_entry_type, sender))
            .await;

        receiver.do_receive().await
    }

    pub async fn initialize(&self) -> ModuleId {
        let (sender, receiver) = oneshot::channel();

        self.sender.do_send(LeaderMessage::Initialize(sender)).await;

        receiver.do_receive().await
    }

    pub async fn append(&self, module_id: ModuleId, bytes: Bytes) {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderMessage::Append(LeaderAppendMessage {
                module_id,
                bytes,
                sender,
            }))
            .await;

        receiver.do_receive().await
    }

    pub async fn load(&self, module_id: ModuleId) -> Result<(), String> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderMessage::Load(module_id, sender))
            .await;

        receiver.do_receive().await.map(|_| ())
    }

    pub async fn dispatch(
        &self,
        module_id: ModuleId,
        payload: Vec<u8>,
        node_id: Option<NodeId>,
    ) -> Result<Vec<u8>, String> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderMessage::Dispatch(LeaderDispatchMessage {
                module_id,
                payload,
                _node_id: node_id,
                sender,
            }))
            .await;

        receiver.do_receive().await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderBuilder {
    sender: mpsc::Sender<LeaderMessage>,
    receiver: mpsc::Receiver<LeaderMessage>,
}

impl LeaderBuilder {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(16);

        Self { sender, receiver }
    }

    pub fn handle(&self) -> LeaderHandle {
        LeaderHandle::new(self.sender.clone())
    }

    pub fn task(self, tcp_listener: TcpListener) -> Result<LeaderTask, Box<dyn Error>> {
        LeaderTask::new(self.sender, self.receiver, tcp_listener)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderTask {
    id: NodeId,
    log: LeaderLog,
    term: Term,
    tcp_listener: TcpListener,
    addresses: HashMap<NodeId, SocketAddr>,
    sender: mpsc::Sender<LeaderMessage>,
    receiver: mpsc::Receiver<LeaderMessage>,
    module_manager: ModuleManager,
    follower_handles: HashMap<NodeId, LeaderFollowerHandle>,
    on_apply_message: HashMap<LogEntryId, LeaderMessage>,
}

impl LeaderTask {
    fn new(
        sender: mpsc::Sender<LeaderMessage>,
        receiver: mpsc::Receiver<LeaderMessage>,
        tcp_listener: TcpListener,
    ) -> Result<Self, Box<dyn Error>> {
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
            follower_handles: HashMap::default(),
            on_apply_message: HashMap::default(),
        })
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
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

                    info!("Connection {} created", address);

                    let (reader, writer) = stream.into_split();

                    LeaderConnectionBuilder::default().task(
                        MessageStreamReader::new(reader),
                        MessageStreamWriter::new(writer),
                        LeaderHandle::new(self.sender.clone()),
                    ).spawn();
                }
                receive_result = self.receiver.recv() => {
                    let message = match receive_result {
                        None => {
                            error!("Nothing received");
                            break
                        }
                        Some(message) => message,
                    };

                    self.on_leader_message(message).await.expect("");
                }
            )
        }
    }

    async fn on_leader_message(&mut self, message: LeaderMessage) -> Result<(), Box<dyn Error>> {
        trace!("{:?}", message);

        match message {
            LeaderMessage::Ping(message) => self.on_ping(message).await,
            LeaderMessage::Register(address, follower_handle, result_sender) => {
                self.register(address, follower_handle, result_sender).await
            }
            LeaderMessage::Replicate(log_entry_type, sender) => {
                let id = self.replicate(log_entry_type).await;

                if sender.send(id).is_err() {
                    error!("Cannot send result");
                }

                Ok(())
            }
            LeaderMessage::Acknowledge(message) => {
                self.on_leader_acknowledge_message(message).await
            }
            LeaderMessage::Initialize(sender) => self.initialize(sender).await,
            LeaderMessage::Append(message) => self.on_leader_append_message(message).await,
            LeaderMessage::Load(id, sender) => self.load(id, sender).await,
            LeaderMessage::Dispatch(message) => self.on_dispatch(message).await,
            LeaderMessage::Handshake {
                node_id,
                address,
                sender,
            } => self.on_handshake(node_id, address, sender).await,
            LeaderMessage::ApplyInitialize {
                log_entry_id,
                sender,
            } => self.on_apply_initialize(log_entry_id, sender).await,
            LeaderMessage::ApplyAppend {
                log_entry_id,
                sender,
            } => self.on_apply_append(log_entry_id, sender).await,
            LeaderMessage::ApplyLoad {
                log_entry_id,
                sender,
            } => self.on_apply_load(log_entry_id, sender).await,
            LeaderMessage::ReplicationStatus(message) => {
                self.on_leader_replication_status_message(message).await
            }
        }
        .expect("");

        Ok(())
    }

    async fn on_apply_initialize(
        &mut self,
        log_entry_id: LogEntryId,
        sender: oneshot::Sender<ModuleId>,
    ) -> Result<(), Box<dyn Error>> {
        let log_entry = match self.log.get(log_entry_id) {
            None => return Err("Log entry should exist".into()),
            Some(log_entry) => log_entry,
        };

        if !matches!(
            log_entry.data().r#type(),
            LogEntryType::Client(ClientLogEntryType::Initialize)
        ) {
            return Err("Log entry type should be Load".into());
        }

        let module_id = match self.module_manager.initialize() {
            Err(error) => {
                error!("{}", error);
                return Ok(());
            }
            Ok(module_id) => module_id,
        };

        sender.do_send(module_id).await;

        Ok(())
    }

    async fn on_apply_append(
        &mut self,
        log_entry_id: LogEntryId,
        sender: oneshot::Sender<()>,
    ) -> Result<(), Box<dyn Error>> {
        let log_entry = match self.log.get(log_entry_id) {
            None => return Err("Log entry should exist".into()),
            Some(log_entry) => log_entry,
        };

        let (module_id, bytes) = match log_entry.data().r#type() {
            LogEntryType::Client(ClientLogEntryType::Append(module_id, bytes)) => {
                (module_id, bytes)
            }
            _ => return Err("Log entry type should be Append".into()),
        };

        self.module_manager
            .append(*module_id, bytes)
            .await
            .expect("");

        sender.do_send(()).await;

        Ok(())
    }

    async fn on_apply_load(
        &mut self,
        log_entry_id: LogEntryId,
        sender: oneshot::Sender<Result<ModuleId, String>>,
    ) -> Result<(), Box<dyn Error>> {
        let log_entry = match self.log.get(log_entry_id) {
            None => return Err("Log entry should exist".into()),
            Some(log_entry) => log_entry,
        };

        let module_id = match log_entry.data().r#type() {
            LogEntryType::Client(ClientLogEntryType::Load(module_id)) => module_id,
            _ => return Err("Log entry type should be Load".into()),
        };

        let result = match self.module_manager.load(*module_id).await {
            Ok(()) => Ok(*module_id),
            Err(error) => Err(error.to_string()),
        };

        sender.do_send(result).await;

        Ok(())
    }

    async fn on_handshake(
        &mut self,
        node_id: NodeId,
        address: SocketAddr,
        sender: oneshot::Sender<Result<(Term, Option<LogEntryId>, LeaderFollowerHandle), String>>,
    ) -> Result<(), Box<dyn Error>> {
        let follower_handle = match self.follower_handles.get(&node_id) {
            Some(follower_handle) => follower_handle.clone(),
            None => {
                sender.send(Err("Follower not found".into())).expect("");

                return Ok(());
            }
        };

        if follower_handle.status().await.available() {
            sender.send(Err("Follower available".into())).expect("");

            return Ok(());
        }

        let mut addresses = self.addresses.clone();
        addresses.insert(node_id, address);

        self
            .replicate(LogEntryType::Cluster(ClusterLogEntryType::Addresses(
                addresses,
            )))
            .await;

        sender
            .do_send(Ok((
                self.term,
                self.log.last_committed_log_entry_id(),
                follower_handle,
            )))
            .await;

        Ok(())
    }

    async fn on_leader_replication_status_message(
        &mut self,
        message: LeaderReplicationStatusMessage,
    ) -> Result<(), Box<dyn Error>> {
        message
            .sender
            .do_send(LeaderReplicationStatus::new(
                self.term,
                self.log.last_committed_log_entry_id(),
            ))
            .await;

        Ok(())
    }

    async fn register(
        &mut self,
        address: SocketAddr,
        follower_handle: LeaderFollowerHandle,
        result_sender: oneshot::Sender<(NodeId, NodeId, Term, HashMap<NodeId, SocketAddr>)>,
    ) -> Result<(), Box<dyn Error>> {
        let id = next_key(self.addresses.keys());
        self.addresses.insert(id, address);

        self.replicate(LogEntryType::Cluster(ClusterLogEntryType::Addresses(
            self.addresses.clone(),
        )))
        .await;

        self.follower_handles.insert(id, follower_handle);

        if result_sender
            .send((id, self.id, self.term, self.addresses.clone()))
            .is_err()
        {
            error!("Cannot send result");
        }

        Ok(())
    }

    async fn replicate(&mut self, log_entry_type: LogEntryType) -> LogEntryId {
        let id = self.log.begin(
            log_entry_type.clone(),
            self.term,
            self.addresses.keys().copied().collect(),
        );

        self.sender
            .do_send(LeaderMessage::Acknowledge(LeaderAcknowledgeMessage {
                acknowledged_log_entry_ids: vec![id],
                missing_log_entry_ids: Vec::default(),
                node_id: self.id,
                sender: None,
            }))
            .await;

        let follower_handles = self.follower_handles.clone();
        let term = self.term;
        let last_committed_log_entry_id = self.log.last_committed_log_entry_id();

        spawn(async move {
            for follower_handle in follower_handles.values() {
                let log_entry_data = LogEntryData::new(id, log_entry_type.clone(), term);

                follower_handle
                    .replicate(term, last_committed_log_entry_id, vec![log_entry_data])
                    .await;
            }
        });

        id
    }

    async fn on_leader_acknowledge_message(
        &mut self,
        message: LeaderAcknowledgeMessage,
    ) -> Result<(), Box<dyn Error>> {
        for log_entry_id in message.acknowledged_log_entry_ids.into_iter() {
            let acknowledgable = match self.log.get(log_entry_id) {
                Some(log_entry) => {
                    log_entry.acknowledging_nodes().contains(&message.node_id)
                        && !log_entry.acknowledged_nodes().contains(&message.node_id)
                }
                None => false,
            };

            if !acknowledgable {
                continue;
            }

            for committed_log_entry_id in self.log.acknowledge(log_entry_id, message.node_id) {
                if let Some(log_entry) = self.log.get(committed_log_entry_id) {
                    match log_entry.data().r#type() {
                        LogEntryType::Client(_) => {
                            self.apply(committed_log_entry_id).await?;
                        }
                        LogEntryType::Cluster(ClusterLogEntryType::Addresses(addresses)) => {
                            self.addresses = addresses.clone();
                            self.apply(committed_log_entry_id).await?;
                        }
                    }
                }
            }
        }

        if let Some(sender) = message.sender {
            sender
                .do_send(
                    message
                        .missing_log_entry_ids
                        .into_iter()
                        .filter_map(|log_entry_id| self.log.get(log_entry_id))
                        .map(|log_entry| log_entry.data().clone())
                        .collect(),
                )
                .await;
        }

        Ok(())
    }

    async fn initialize(
        &mut self,
        sender: oneshot::Sender<ModuleId>,
    ) -> Result<(), Box<dyn Error>> {
        let log_entry_id = self
            .replicate(LogEntryType::Client(ClientLogEntryType::Initialize))
            .await;

        self.on_apply_message.insert(
            log_entry_id,
            LeaderMessage::ApplyInitialize {
                log_entry_id,
                sender,
            },
        );

        Ok(())
    }

    async fn on_leader_append_message(
        &mut self,
        message: LeaderAppendMessage,
    ) -> Result<(), Box<dyn Error>> {
        let log_entry_id = self
            .replicate(LogEntryType::Client(ClientLogEntryType::Append(
                message.module_id,
                message.bytes,
            )))
            .await;

        self.on_apply_message.insert(
            log_entry_id,
            LeaderMessage::ApplyAppend {
                log_entry_id,
                sender: message.sender,
            },
        );

        Ok(())
    }

    async fn apply(&mut self, log_entry_id: LogEntryId) -> Result<(), Box<dyn Error>> {
        debug!("Apply {}", log_entry_id);

        if let Some(message) = self.on_apply_message.remove(&log_entry_id) {
            self.sender.do_send(message).await;
        }

        Ok(())
    }

    async fn load(
        &mut self,
        id: ModuleId,
        sender: oneshot::Sender<Result<ModuleId, String>>,
    ) -> Result<(), Box<dyn Error>> {
        let log_entry_id = self
            .replicate(LogEntryType::Client(ClientLogEntryType::Load(id)))
            .await;

        self.on_apply_message.insert(
            log_entry_id,
            LeaderMessage::ApplyLoad {
                log_entry_id,
                sender,
            },
        );

        Ok(())
    }

    async fn on_ping(&mut self, message: LeaderPingMessage) -> Result<(), Box<dyn Error>> {
        message.sender.do_send(()).await;

        Ok(())
    }

    async fn on_dispatch(&mut self, message: LeaderDispatchMessage) -> Result<(), Box<dyn Error>> {
        let module = match self.module_manager.get(message.module_id) {
            Some(module) => module.clone(),
            None => {
                message.sender.do_send(Err("Module not found".into())).await;

                return Ok(());
            }
        };

        spawn(async move {
            let result = module
                .event_handler()
                .dispatch(
                    tokio::runtime::Handle::current(),
                    DispatchModuleEventInput::new(message.payload),
                )
                .await
                .map(|output| {
                    let (payload,) = output.into();

                    payload
                })
                .map_err(|e| e.to_string());

            message.sender.do_send(result).await;
        });

        Ok(())
    }
}
