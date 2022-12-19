pub mod client;
pub mod connection;
pub mod follower;
pub mod log;

////////////////////////////////////////////////////////////////////////////////////////////////////

use connection::LeaderConnection;
use follower::LeaderFollowerMessage;
use futures::future::BoxFuture;
use futures::FutureExt;
use log::LeaderLog;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::{error, trace};
use zlambda_common::algorithm::next_key;
use zlambda_common::log::{
    ClientLogEntryType, ClusterLogEntryType, LogEntryData, LogEntryId, LogEntryType,
};
use zlambda_common::message::{MessageStreamReader, MessageStreamWriter};
use zlambda_common::module::{DispatchModuleEventInput, ModuleId, ModuleManager};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum LeaderMessage {
    Register(
        SocketAddr,
        mpsc::Sender<LeaderFollowerMessage>,
        oneshot::Sender<(NodeId, NodeId, Term, HashMap<NodeId, SocketAddr>)>,
    ),
    Acknowledge(Vec<LogEntryId>, NodeId),
    Replicate(LogEntryType, oneshot::Sender<LogEntryId>),
    Initialize(oneshot::Sender<ModuleId>),
    Append(ModuleId, Vec<u8>, oneshot::Sender<()>),
    Load(ModuleId, oneshot::Sender<Result<ModuleId, String>>),
    Dispatch(ModuleId, Vec<u8>, oneshot::Sender<Result<Vec<u8>, String>>),
    Handshake {
        node_id: NodeId,
        address: SocketAddr,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        result: oneshot::Sender<Result<(), String>>,
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
    on_load_handler:
        HashMap<LogEntryId, Box<dyn FnOnce(Result<ModuleId, String>) -> BoxFuture<'static, ()>>>,
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
            on_load_handler: HashMap::default(),
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

                    self.on_message(message).await.expect("");
                }
            )
        }
    }

    async fn on_message(
        &mut self,
        message: LeaderMessage,
    ) -> Result<(), Box<dyn Error>> {
        match message {
            LeaderMessage::Register(address, follower_sender, result_sender) => self.register(
                address,
                follower_sender,
                result_sender,
            ).await,
            LeaderMessage::Replicate(log_entry_type, sender) => {
                let id = self.replicate(log_entry_type).await;

                if sender.send(id).is_err() {
                    error!("Cannot send result");
                }

                Ok(())
            }
            LeaderMessage::Acknowledge(log_entry_ids, node_id) => self.acknowledge(log_entry_ids, node_id).await,
            LeaderMessage::Initialize(sender)  => self.initialize(sender).await,
            LeaderMessage::Append(id, chunk, sender) => self.append(id, chunk, sender).await,
            LeaderMessage::Load(id, sender) => self.load(id, sender).await,
            LeaderMessage::Dispatch(id, payload, sender) => self.dispatch(id, payload, sender).await,
            LeaderMessage::Handshake {
                node_id,
            address,
            reader,
            writer,
            result,
            } => self.on_handshake(node_id,
            address,
            reader,
            writer,
            result,).await
        }.expect("");

        Ok(())
    }

    async fn on_handshake(
        &mut self,
        node_id: NodeId,
        address: SocketAddr,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        result: oneshot::Sender<Result<(), String>>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn register(
        &mut self,
        address: SocketAddr,
        follower_sender: mpsc::Sender<LeaderFollowerMessage>,
        result_sender: oneshot::Sender<(NodeId, NodeId, Term, HashMap<NodeId, SocketAddr>)>,
    ) -> Result<(), Box<dyn Error>> {
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

        Ok(())
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
                .send(LeaderFollowerMessage::Replicate(
                    self.term,
                    self.log.last_committed_log_entry_id(),
                    vec![log_entry_data],
                    sender,
                ))
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
            .send(LeaderMessage::Acknowledge(vec![id], self.id))
            .await
            .expect("Cannot send");

        id
    }

    async fn acknowledge(&mut self, log_entry_ids: Vec<LogEntryId>, node_id: NodeId

    ) -> Result<(), Box<dyn Error>> {
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
                                .await?;
                        }
                        LogEntryType::Cluster(ClusterLogEntryType::Addresses(addresses)) => {
                            self.addresses = addresses.clone();
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn initialize(&mut self, sender: oneshot::Sender<ModuleId>
    ) -> Result<(), Box<dyn Error>> {
        let id = self
            .replicate(LogEntryType::Client(ClientLogEntryType::Initialize))
            .await;

        self.on_apply_initialize_module_handler.insert(
            id,
            Box::new(|module_id| {
                {
                    async move {
                        if let Err(error) = sender.send(module_id) {
                            error!("{}", error);
                        }
                    }
                }
                .boxed()
            }),
        );

        Ok(())
    }

    async fn append(&mut self, id: ModuleId, chunk: Vec<u8>, sender: oneshot::Sender<()>
    ) -> Result<(), Box<dyn Error>> {
        self.replicate(LogEntryType::Client(ClientLogEntryType::Append(id, chunk)))
            .await;

        if sender.send(()).is_err() {
            error!("Error sending append module");
        }

    Ok(())
    }

    async fn apply(&mut self, log_entry_id: LogEntryId, client_log_entry_type: ClientLogEntryType
    ) -> Result<(), Box<dyn Error>> {
        trace!("Apply {}", log_entry_id);

        match client_log_entry_type {
            ClientLogEntryType::Initialize => {
                let module_id = match self.module_manager.initialize() {
                    Err(error) => {
                        error!("{}", error);
                        return Ok(());
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
            ClientLogEntryType::Append(id, chunk) => {
                if let Err(error) = self.module_manager.append(id, &chunk).await {
                    error!("{}", error);
                }
            }
            ClientLogEntryType::Load(id) => {
                let result = match self.module_manager.load(id).await {
                    Ok(()) => Ok(id),
                    Err(error) => Err(error.to_string()),
                };

                if let Some(handler) = self.on_load_handler.remove(&log_entry_id) {
                    handler(result).await;
                }
            }
        };

        Ok(())
    }

    async fn load(&mut self, id: ModuleId, sender: oneshot::Sender<Result<ModuleId, String>>
    ) -> Result<(), Box<dyn Error>> {
        let id = self
            .replicate(LogEntryType::Client(ClientLogEntryType::Load(id)))
            .await;

        self.on_load_handler.insert(
            id,
            Box::new(|result| {
                {
                    async move {
                        if sender.send(result).is_err() {
                            error!("Cannot send");
                        }
                    }
                }
                .boxed()
            }),
        );

        Ok(())
    }

    async fn dispatch(
        &mut self,
        id: ModuleId,
        payload: Vec<u8>,
        sender: oneshot::Sender<Result<Vec<u8>, String>>,
    ) -> Result<(), Box<dyn Error>> {
        let module = match self.module_manager.get(id) {
            Some(module) => module.clone(),
            None => {
                sender
                    .send(Err("Module not found".into()))
                    .expect("Cannot send");
                return Ok(());
            }
        };

        spawn(async move {
            let result = module
                .event_handler()
                .dispatch(
                    tokio::runtime::Handle::current(),
                    DispatchModuleEventInput::new(payload),
                )
                .await
                .map(|output| {
                    let (payload,) = output.into();

                    payload
                })
                .map_err(|e| e.to_string());

            sender.send(result).expect("Cannot send");
        });

        Ok(())
    }
}
