use crate::common::deserialize::deserialize_from_bytes;
use crate::common::message::DoSend;
use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender, SynchronizableMessageOutputSender,
};
use crate::common::module::ModuleManager;
use crate::common::net::{TcpListener, TcpStream, ToSocketAddrs};
use crate::common::runtime::{select, spawn};
use crate::common::serialize::serialize_to_bytes;
use crate::common::sync::mpsc;
use crate::common::task::JoinHandle;
use crate::general::{
    GeneralMessage, GeneralRecoveryRequestMessage, GeneralRecoveryRequestMessageInput,
    GeneralRecoveryResponseMessageInput, GeneralRegistrationRequestMessage,
    GeneralRegistrationRequestMessageInput, GeneralRegistrationResponseMessageInput,
};
use crate::server::client::{ServerClientId, ServerClientMessage, ServerClientTask};
use crate::server::connection::ServerConnectionTask;
use crate::server::node::{
    ServerNodeLogAppendInitiateMessageInput, ServerNodeLogAppendResponseMessageInput,
    ServerNodeLogEntriesCommitMessageInput,
};
use crate::server::{
    AddServerLogEntryData, Log, LogEntryData, LogEntryId, LogEntryIssueId, LogEntryIssuer,
    LogError, LogFollowerType, LogId, LogLeaderType, LogManager, LogSystemIssuer, LogType,
    NewServerError, Server, ServerClientGetMessage, ServerClientGetMessageOutput,
    ServerClientRegistrationMessage, ServerClientResignationMessage, ServerCommitCommitMessage,
    ServerCommitCommitMessageInput, ServerCommitLogCreateMessage,
    ServerCommitLogCreateMessageInput, ServerCommitMessage, ServerCommitRegistrationMessage,
    ServerCommitRegistrationMessageInput, ServerId, ServerLeaderServerIdGetMessage,
    ServerLeaderServerIdGetMessageOutput, ServerLogAppendInitiateMessage,
    ServerLogAppendInitiateMessageOutput, ServerLogAppendRequestMessage, ServerLogCreateMessage,
    ServerLogCreateMessageOutput, ServerLogEntriesAcknowledgementMessage,
    ServerLogEntriesCommitMessage, ServerLogEntriesGetMessage, ServerLogEntriesGetMessageOutput,
    ServerLogEntriesRecoveryMessage, ServerLogEntriesReplicationMessage,
    ServerLogEntriesReplicationMessageOutput, ServerMessage, ServerModule,
    ServerModuleCommitEventInput, ServerModuleGetMessage, ServerModuleGetMessageOutput,
    ServerModuleLoadMessage, ServerModuleLoadMessageOutput, ServerModuleShutdownEventInput,
    ServerModuleStartupEventInput, ServerModuleUnloadMessage, ServerModuleUnloadMessageOutput,
    ServerNodeMessage, ServerNodeReplicationMessage, ServerNodeReplicationMessageInput,
    ServerNodeTask, ServerRecoveryMessage, ServerRecoveryMessageNotALeaderOutput,
    ServerRecoveryMessageOutput, ServerRecoveryMessageSuccessOutput, ServerRegistrationMessage,
    ServerRegistrationMessageNotALeaderOutput, ServerRegistrationMessageSuccessOutput,
    ServerServerIdGetMessage, ServerServerIdGetMessageOutput,
    ServerServerNodeMessageSenderGetAllMessage, ServerServerNodeMessageSenderGetAllMessageOutput,
    ServerServerNodeMessageSenderGetMessage, ServerServerNodeMessageSenderGetMessageOutput,
    ServerServerSocketAddressGetMessage, ServerServerSocketAddressGetMessageOutput,
    ServerServerSocketAddressesGetMessage, ServerServerSocketAddressesGetMessageOutput,
    ServerSocketAcceptMessage, ServerSocketAcceptMessageInput, ServerSystemCreateLogLogEntryData,
    ServerSystemLogEntryData, SERVER_SYSTEM_LOG_ID,
};
use async_recursion::async_recursion;
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error, info};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct CommitTaskMessage {
    module: Arc<dyn ServerModule>,
    input: ServerModuleCommitEventInput,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct CommitTask {
    receiver: mpsc::Receiver<CommitTaskMessage>,
}

impl CommitTask {
    async fn run(&mut self) {
        while let Some(message) = self.receiver.recv().await {
            message.module.on_commit(message.input).await;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct ServerTask {
    server_id: ServerId,
    leader_server_id: ServerId,
    tcp_listener: TcpListener,
    sender: MessageQueueSender<ServerMessage>,
    receiver: MessageQueueReceiver<ServerMessage>,
    commit_messages: HashMap<LogEntryId, Vec<ServerMessage>>,
    module_manager: ModuleManager<dyn ServerModule>,
    server_client_message_senders: Vec<Option<MessageQueueSender<ServerClientMessage>>>,
    server_node_message_senders: Vec<Option<MessageQueueSender<ServerNodeMessage>>>,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    server: Arc<Server>,
    running: bool,
    log_manager: LogManager,
    #[allow(dead_code)]
    commit_tasks: JoinHandle<()>,
    commit_sender: mpsc::Sender<CommitTaskMessage>,
    current_log_entry_issue_id: LogEntryIssueId,
    commit_issue_senders: HashMap<LogEntryIssueId, SynchronizableMessageOutputSender>,
}

impl ServerTask {
    pub async fn new<S, T>(
        listener_address: S,
        follower_data: Option<(T, Option<ServerId>)>,
        modules: impl Iterator<Item = Arc<dyn ServerModule>>,
    ) -> Result<Self, NewServerError>
    where
        S: ToSocketAddrs + Debug,
        T: ToSocketAddrs + Debug,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;
        let local_addr = tcp_listener.local_addr()?;
        let current_log_entry_issue_id = LogEntryIssueId::from(0);
        let commit_issue_senders = HashMap::default();

        let (queue_sender, queue_receiver) = message_queue();
        let mut module_manager = ModuleManager::default();

        for module in modules {
            module_manager.load(module)?;
        }

        let server = Server::new(queue_sender.clone());

        let (commit_sender, commit_receiver) = mpsc::channel(256);

        let commit_tasks = spawn(async move {
            (CommitTask {
                receiver: commit_receiver,
            })
            .run()
            .await
        });

        match follower_data {
            None => Ok(Self {
                server_id: ServerId::default(),
                leader_server_id: ServerId::default(),
                tcp_listener,
                sender: queue_sender.clone(),
                receiver: queue_receiver,
                commit_messages: HashMap::default(),
                module_manager,
                server_client_message_senders: Vec::default(),
                server_node_message_senders: Vec::default(),
                server_socket_addresses: vec![Some(local_addr)],
                running: true,
                server,
                log_manager: (|| {
                    let mut log_manager = LogManager::default();
                    log_manager.insert(Log::new(
                        LogId::from(0),
                        LogLeaderType::default().into(),
                        Some(LogSystemIssuer::new().into()),
                    ));

                    log_manager
                })(),
                commit_tasks,
                commit_sender,
                current_log_entry_issue_id,
                commit_issue_senders,
            }),
            Some((registration_address, None)) => {
                let address = tcp_listener.local_addr()?;
                let mut socket = TcpStream::connect(registration_address).await?;

                let (
                    server_id,
                    leader_server_id,
                    server_socket_addresses,
                    term,
                    socket_sender,
                    socket_receiver,
                ) = loop {
                    let (reader, writer) = socket.into_split();

                    let (mut sender, mut receiver) = (
                        MessageSocketSender::<GeneralMessage>::new(writer),
                        MessageSocketReceiver::<GeneralMessage>::new(reader),
                    );

                    sender
                        .send(GeneralRegistrationRequestMessage::new(
                            GeneralRegistrationRequestMessageInput::new(address),
                        ))
                        .await?;

                    match receiver.receive().await? {
                        None => return Err(MessageError::ExpectedMessage.into()),
                        Some(GeneralMessage::RegistrationResponse(message)) => {
                            let (input,) = message.into();

                            match input {
                                GeneralRegistrationResponseMessageInput::NotALeader(input) => {
                                    socket =
                                        TcpStream::connect(input.leader_server_socket_address())
                                            .await?;
                                    continue;
                                }
                                GeneralRegistrationResponseMessageInput::Success(input) => {
                                    let (
                                        server_id,
                                        leader_server_id,
                                        server_socket_addresses,
                                        term,
                                    ) = input.into();

                                    break (
                                        server_id,
                                        leader_server_id,
                                        server_socket_addresses,
                                        term,
                                        sender,
                                        receiver,
                                    );
                                }
                            }
                        }
                        Some(message) => {
                            return Err(
                                MessageError::UnexpectedMessage(format!("{message:?}")).into()
                            )
                        }
                    }
                };

                info!(
                    "Registered as server {} at leader {} with term {}",
                    server_id, leader_server_id, term
                );

                let max_id = max(usize::from(leader_server_id), usize::from(server_id));
                let mut server_node_message_senders = Vec::default();

                let leader_node_task = ServerNodeTask::new(
                    leader_server_id,
                    queue_sender.clone(),
                    Some((socket_sender, socket_receiver)),
                    server.clone(),
                );

                server_node_message_senders.resize_with(max_id + 1, || None);
                if let Some(sender) =
                    server_node_message_senders.get_mut(usize::from(leader_server_id))
                {
                    *sender = Some(leader_node_task.sender().clone());
                }

                leader_node_task.spawn();

                Ok(Self {
                    server_id,
                    leader_server_id,
                    tcp_listener,
                    sender: queue_sender,
                    receiver: queue_receiver,
                    commit_messages: HashMap::default(),
                    module_manager,
                    server_client_message_senders: Vec::default(),
                    server_node_message_senders,
                    server_socket_addresses,
                    running: true,
                    server,
                    log_manager: (|| {
                        let mut log_manager = LogManager::default();
                        log_manager.insert(Log::new(
                            LogId::from(0),
                            LogFollowerType::default().into(),
                            Some(LogSystemIssuer::new().into()),
                        ));

                        log_manager
                    })(),
                    commit_tasks,
                    commit_sender,
                    current_log_entry_issue_id,
                    commit_issue_senders,
                })
            }
            Some((recovery_address, Some(server_id))) => {
                let mut socket = TcpStream::connect(recovery_address).await?;

                let (
                    leader_server_id,
                    server_socket_addresses,
                    term,
                    socket_sender,
                    socket_receiver,
                ) = loop {
                    let (reader, writer) = socket.into_split();

                    let (mut sender, mut receiver) = (
                        MessageSocketSender::<GeneralMessage>::new(writer),
                        MessageSocketReceiver::<GeneralMessage>::new(reader),
                    );

                    sender
                        .send(GeneralRecoveryRequestMessage::new(
                            GeneralRecoveryRequestMessageInput::new(server_id),
                        ))
                        .await?;

                    match receiver.receive().await? {
                        None => return Err(MessageError::ExpectedMessage.into()),
                        Some(GeneralMessage::RecoveryResponse(message)) => {
                            let (input,) = message.into();

                            match input {
                                GeneralRecoveryResponseMessageInput::NotALeader(input) => {
                                    socket =
                                        TcpStream::connect(input.leader_socket_address()).await?;
                                    continue;
                                }
                                GeneralRecoveryResponseMessageInput::IsOnline => {
                                    return Err(NewServerError::IsOnline(server_id));
                                }
                                GeneralRecoveryResponseMessageInput::Unknown => {
                                    return Err(NewServerError::Unknown(server_id));
                                }
                                GeneralRecoveryResponseMessageInput::Success(input) => {
                                    let (leader_server_id, server_socket_addresses, term) =
                                        input.into();

                                    break (
                                        leader_server_id,
                                        server_socket_addresses,
                                        term,
                                        sender,
                                        receiver,
                                    );
                                }
                            }
                        }
                        Some(message) => {
                            return Err(
                                MessageError::UnexpectedMessage(format!("{message:?}")).into()
                            )
                        }
                    }
                };

                info!("Recovered at leader {} and term {}", leader_server_id, term);

                let max_id = max(usize::from(leader_server_id), usize::from(server_id));
                let mut server_node_message_senders = Vec::default();

                let leader_node_task = ServerNodeTask::new(
                    leader_server_id,
                    queue_sender.clone(),
                    Some((socket_sender, socket_receiver)),
                    server.clone(),
                );

                server_node_message_senders.resize_with(max_id + 1, || None);
                if let Some(sender) =
                    server_node_message_senders.get_mut(usize::from(leader_server_id))
                {
                    *sender = Some(leader_node_task.sender().clone());
                }

                leader_node_task.spawn();

                Ok(Self {
                    server_id,
                    leader_server_id,
                    tcp_listener,
                    sender: queue_sender,
                    receiver: queue_receiver,
                    commit_messages: HashMap::default(),
                    module_manager,
                    server_client_message_senders: Vec::default(),
                    server_node_message_senders,
                    server_socket_addresses,
                    running: true,
                    server,
                    log_manager: (|| {
                        let mut log_manager = LogManager::default();
                        log_manager.insert(Log::new(
                            LogId::from(0),
                            LogFollowerType::default().into(),
                            Some(LogSystemIssuer::new().into()),
                        ));

                        log_manager
                    })(),
                    commit_tasks,
                    commit_sender,
                    current_log_entry_issue_id,
                    commit_issue_senders,
                })
            }
        }
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        for (module_id, module) in self.module_manager.iter() {
            let module = module.clone();
            let sender = self.sender.clone();

            spawn(async move {
                module
                    .on_startup(ServerModuleStartupEventInput::new(
                        Arc::new(Server::from(sender)),
                        module_id,
                    ))
                    .await;
            });
        }

        while self.running {
            select!(
                result = self.tcp_listener.accept() => {
                    let (socket_stream, socket_address) = match result {
                        Ok(socket) => socket,
                        Err(error) => {
                            error!("{}", error);
                            continue;
                        }
                    };

                    self.on_server_message(
                        ServerSocketAcceptMessage::new(
                            ServerSocketAcceptMessageInput::new(socket_stream, socket_address),
                        ).into(),
                    ).await;
                }
                message = self.receiver.do_receive() => {
                    self.on_server_message(message).await;
                }
            )
        }

        for (module_id, module) in self.module_manager.iter() {
            module
                .on_shutdown(ServerModuleShutdownEventInput::new(
                    self.server.clone(),
                    module_id,
                ))
                .await;
        }
    }

    #[async_recursion]
    async fn on_server_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::Ping => {}
            ServerMessage::SocketAccept(message) => {
                self.on_server_socket_accept_message(message).await
            }
            ServerMessage::Registration(message) => {
                self.on_server_registration_message(message).await
            }
            ServerMessage::CommitRegistration(message) => {
                self.on_server_commit_registration_message(message).await
            }
            ServerMessage::Recovery(message) => self.on_server_recovery_message(message).await,
            ServerMessage::LogEntriesReplication(message) => {
                self.on_server_log_entries_replication_message(message)
                    .await
            }
            ServerMessage::LogEntriesAcknowledgement(message) => {
                self.on_server_log_entries_acknowledgement_message(message)
                    .await
            }
            ServerMessage::LogEntriesRecovery(message) => {
                self.on_server_log_entries_recovery_message(message).await
            }
            ServerMessage::LogAppendRequest(message) => {
                self.on_server_log_append_request_message(message).await
            }
            ServerMessage::LogAppendInitiate(message) => {
                self.on_server_log_append_initiate_message(message).await
            }
            ServerMessage::LogEntriesGet(message) => {
                self.on_server_log_entries_get_message(message).await
            }
            ServerMessage::LogEntriesCommit(message) => {
                self.on_server_log_entries_commit_message(message).await
            }
            ServerMessage::ModuleGet(message) => self.on_server_module_get_message(message).await,
            ServerMessage::ModuleLoad(message) => self.on_server_module_load_message(message).await,
            ServerMessage::ModuleUnload(message) => {
                self.on_server_module_unload_message(message).await
            }
            ServerMessage::ClientRegistration(message) => {
                self.on_server_client_registration(message).await
            }
            ServerMessage::ClientResignation(message) => {
                self.on_server_client_resignation(message).await
            }
            ServerMessage::ServerSocketAddressGet(message) => {
                self.on_server_server_socket_address_get_message(message)
                    .await
            }
            ServerMessage::ServerIdGet(message) => {
                self.on_server_server_id_get_message(message).await
            }
            ServerMessage::LeaderServerIdGet(message) => {
                self.on_server_leader_server_id_get_message(message).await
            }
            ServerMessage::ServerNodeMessageSenderGet(message) => {
                self.on_server_server_node_message_sender_get_message(message)
                    .await
            }
            ServerMessage::ServerNodeMessageSenderGetAll(message) => {
                self.on_server_server_node_message_sender_get_all_message(message)
                    .await
            }
            ServerMessage::Commit(message) => self.on_server_commit_message(message).await,
            ServerMessage::CommitCommit(message) => {
                self.on_server_commit_commit_message(message).await
            }
            ServerMessage::ServerSocketAddressesGet(message) => {
                self.on_server_server_socket_addresses_get_message(message)
                    .await
            }
            ServerMessage::LogCreate(message) => self.on_server_log_create_message(message).await,
            ServerMessage::CommitLogCreate(message) => {
                self.on_server_commit_log_create_message(message).await
            }
            ServerMessage::ServerClientGet(message) => {
                self.on_server_client_get_message(message).await
            }
        }
    }

    async fn on_server_socket_accept_message(&mut self, message: ServerSocketAcceptMessage) {
        let (input,) = message.into();
        let (socket_stream, socket_address) = input.into();

        info!("Connection from {}", socket_address);

        let (reader, writer) = socket_stream.into_split();

        ServerConnectionTask::new(
            self.sender.clone(),
            MessageSocketSender::new(writer),
            MessageSocketReceiver::new(reader),
        )
        .spawn()
    }

    async fn on_server_registration_message(&mut self, message: ServerRegistrationMessage) {
        let (input, output_sender) = message.into();

        if self.leader_server_id == self.server_id {
            let node_server_id = {
                let mut iterator = self.server_socket_addresses.iter();
                let mut index = 0;

                loop {
                    if iterator.next().is_none() {
                        break index;
                    }

                    index += 1
                }
            };

            if node_server_id >= self.server_socket_addresses.len() {
                self.server_socket_addresses
                    .resize_with(node_server_id + 1, || None);
            }

            if node_server_id >= self.server_node_message_senders.len() {
                self.server_node_message_senders
                    .resize_with(node_server_id + 1, || None);
            }

            let task = ServerNodeTask::new(
                node_server_id.into(),
                self.sender.clone(),
                None,
                self.server.clone(),
            );
            let sender = task.sender().clone();
            task.spawn();

            if let Some(server_socket_address) =
                self.server_socket_addresses.get_mut(node_server_id)
            {
                *server_socket_address = Some(input.server_socket_address());
            }

            if let Some(server_node_message_sender) =
                self.server_node_message_senders.get_mut(node_server_id)
            {
                *server_node_message_sender = Some(sender);
            }

            let log_entry_ids = self
                .replicate(
                    SERVER_SYSTEM_LOG_ID,
                    vec![(
                        serialize_to_bytes(&ServerSystemLogEntryData::from(
                            AddServerLogEntryData::new(
                                node_server_id.into(),
                                input.server_socket_address(),
                            ),
                        ))
                        .expect("")
                        .freeze(),
                        None,
                    )],
                )
                .await;

            self.commit_messages
                .entry(*log_entry_ids.first().expect("valid log entry id"))
                .or_insert(Vec::default())
                .push(
                    ServerCommitRegistrationMessage::new(
                        ServerCommitRegistrationMessageInput::new(node_server_id.into()),
                        output_sender,
                    )
                    .into(),
                );

            self.acknowledge(SERVER_SYSTEM_LOG_ID, &log_entry_ids, self.server_id)
                .await;
        } else {
            let leader_server_socket_address = match self
                .server_socket_addresses
                .get(usize::from(self.leader_server_id))
            {
                Some(Some(leader_server_socket_address)) => leader_server_socket_address,
                None | Some(None) => {
                    panic!("Expected leader server socket address");
                }
            };

            output_sender
                .do_send(ServerRegistrationMessageNotALeaderOutput::new(
                    *leader_server_socket_address,
                ))
                .await;
        }
    }

    async fn on_server_commit_registration_message(
        &mut self,
        message: ServerCommitRegistrationMessage,
    ) {
        let (input, output_sender) = message.into();

        let log = self
            .log_manager
            .get(SERVER_SYSTEM_LOG_ID)
            .expect("valid server system log");

        let server_node_message_sender = match self
            .server_node_message_senders
            .get(usize::from(input.node_server_id()))
        {
            Some(Some(server_node_message_sender)) => server_node_message_sender.clone(),
            None | Some(None) => panic!("Server node {} should exist", input.node_server_id()),
        };

        output_sender
            .do_send(ServerRegistrationMessageSuccessOutput::new(
                input.node_server_id(),
                self.server_id,
                self.server_socket_addresses.clone(),
                log.last_committed_log_entry_id(),
                log.current_term(),
                server_node_message_sender,
            ))
            .await;
    }

    async fn on_server_recovery_message(&mut self, message: ServerRecoveryMessage) {
        let (input, sender) = message.into();

        if self.server_id != self.leader_server_id {
            let leader_server_socket_address = match self
                .server_socket_addresses
                .get(usize::from(self.leader_server_id))
            {
                Some(Some(leader_server_socket_address)) => *leader_server_socket_address,
                None | Some(None) => {
                    panic!("Expected leader server socket address");
                }
            };

            sender
                .do_send(ServerRecoveryMessageNotALeaderOutput::new(
                    leader_server_socket_address,
                ))
                .await;

            return;
        }

        match self
            .server_node_message_senders
            .get(usize::from(input.server_id()))
        {
            None | Some(None) => sender.do_send(ServerRecoveryMessageOutput::Unknown).await,
            Some(Some(server_node_message_senders)) => {
                let log = self
                    .log_manager
                    .get(SERVER_SYSTEM_LOG_ID)
                    .expect("server system log");

                sender
                    .do_send(ServerRecoveryMessageSuccessOutput::new(
                        self.server_id,
                        self.server_socket_addresses.clone(),
                        log.last_committed_log_entry_id(),
                        log.current_term(),
                        server_node_message_senders.clone(),
                    ))
                    .await;
            }
        }
    }

    async fn on_server_log_entries_replication_message(
        &mut self,
        message: ServerLogEntriesReplicationMessage,
    ) {
        let (input, sender) = message.into();
        let (log_entries_data,) = input.into();

        sender
            .do_send(ServerLogEntriesReplicationMessageOutput::new(
                self.replicate(
                    SERVER_SYSTEM_LOG_ID,
                    log_entries_data
                        .into_iter()
                        .map(|data| (data, None))
                        .collect(),
                )
                .await,
            ))
            .await;
    }

    async fn on_server_log_entries_acknowledgement_message(
        &mut self,
        message: ServerLogEntriesAcknowledgementMessage,
    ) {
        let (input,) = message.into();

        self.acknowledge(input.log_id(), input.log_entry_ids(), input.server_id())
            .await;
    }

    async fn on_server_log_entries_recovery_message(
        &mut self,
        message: ServerLogEntriesRecoveryMessage,
    ) {
        let leader_log = match self
            .log_manager
            .get(SERVER_SYSTEM_LOG_ID)
            .expect("existing server system log")
            .r#type()
        {
            LogType::Leader(leader) => leader,
            LogType::Follower(_follower) => panic!("Log must be leader"),
        };

        let (input,) = message.into();

        let log_entries = input
            .log_entry_ids()
            .iter()
            .filter_map(|log_entry_id| leader_log.entries().get(usize::from(*log_entry_id)))
            .chain(leader_log.acknowledgeable_log_entries_for_server(input.server_id()))
            .cloned()
            .collect::<Vec<_>>();

        let server_node_message_sender = match self
            .server_node_message_senders
            .get(usize::from(input.server_id()))
        {
            Some(Some(server_node_message_sender)) => server_node_message_sender,
            None | Some(None) => {
                panic!("Node does not exist");
            }
        };

        server_node_message_sender
            .do_send(ServerNodeReplicationMessage::new(
                ServerNodeReplicationMessageInput::new(
                    SERVER_SYSTEM_LOG_ID,
                    log_entries,
                    leader_log.last_committed_log_entry_id(),
                    leader_log.current_term(),
                ),
            ))
            .await;
    }

    async fn on_server_log_entries_get_message(&mut self, message: ServerLogEntriesGetMessage) {
        let (input, output_sender) = message.into();

        let log_entry = self
            .log_manager
            .get(input.log_id())
            .map(|log| log.entries().get(input.log_entry_id()))
            .flatten()
            .cloned();

        output_sender
            .do_send(ServerLogEntriesGetMessageOutput::new(log_entry))
            .await;
    }

    async fn on_server_log_entries_commit_message(
        &mut self,
        message: ServerLogEntriesCommitMessage,
    ) {
        let (input,) = message.into();
        let (log_id, log_entry_data, log_entry_issuer) = input.into();

        let log_entry_id = match self
            .replicate(log_id, vec![(log_entry_data, Some(log_entry_issuer))])
            .await
            .into_iter()
            .next()
        {
            Some(log_entry_id) => log_entry_id,
            None => return,
        };

        self.acknowledge(log_id, &vec![log_entry_id], self.server_id)
            .await;
    }

    async fn on_server_log_append_request_message(
        &mut self,
        message: ServerLogAppendRequestMessage,
    ) {
        let (input,) = message.into();
        let (server_id, log_id, log_entries, last_committed_log_entry_id, log_current_term) =
            input.into();

        let follower_log = match self
            .log_manager
            .get_mut(log_id)
            .expect("existing server system log")
            .type_mut()
        {
            LogType::Leader(_leader) => panic!("Cannot append entries to leader node"),
            LogType::Follower(follower) => follower,
        };

        let mut log_entry_ids = Vec::with_capacity(log_entries.len());

        for log_entry in log_entries.into_iter() {
            log_entry_ids.push(log_entry.id());
            follower_log.push(log_entry);
        }

        let (missing_log_entry_ids, committed_log_entry_id_range) =
            match last_committed_log_entry_id {
                Some(last_committed_log_entry_id) => {
                    let from_last_committed_log_entry_id =
                        follower_log.last_committed_log_entry_id().map(usize::from);

                    let missing_log_entry_ids =
                        follower_log.commit(last_committed_log_entry_id, log_current_term);

                    let to_last_committed_log_entry_id =
                        follower_log.last_committed_log_entry_id().map(usize::from);

                    let committed_log_entry_id_range = match (
                        from_last_committed_log_entry_id,
                        to_last_committed_log_entry_id,
                    ) {
                        (None, Some(to)) => Some(0..(to + 1)),
                        (Some(from), Some(to)) if from < to => Some((from + 1)..(to + 1)),
                        _ => None,
                    };

                    (missing_log_entry_ids, committed_log_entry_id_range)
                }
                None => (Vec::default(), None),
            };

        if !log_entry_ids.is_empty() || !missing_log_entry_ids.is_empty() {
            if let Some(Some(sender)) = self.server_node_message_senders.get(usize::from(server_id))
            {
                sender
                    .do_send_asynchronous(ServerNodeLogAppendResponseMessageInput::new(
                        log_id,
                        log_entry_ids,
                        missing_log_entry_ids,
                    ))
                    .await;
            }
        }

        if let Some(committed_log_entry_id_range) = committed_log_entry_id_range {
            self.on_commit_log_entries(log_id, committed_log_entry_id_range.map(LogEntryId::from))
                .await;
        }
    }

    async fn on_server_log_append_initiate_message(
        &mut self,
        message: ServerLogAppendInitiateMessage,
    ) {
        let (input, output_sender) = message.into();

        let log = self.log_manager.get(input.log_id()).expect("existing log");

        output_sender
            .do_send(ServerLogAppendInitiateMessageOutput::new(
                log.last_committed_log_entry_id(),
                log.current_term(),
                match log.r#type() {
                    LogType::Leader(leader) => leader.entries().iter().cloned().collect(),
                    LogType::Follower(_follower) => panic!("Node must be leader"),
                },
            ))
            .await;
    }

    async fn on_server_module_get_message(&mut self, message: ServerModuleGetMessage) {
        let (input, sender) = message.into();

        sender
            .do_send(ServerModuleGetMessageOutput::new(
                self.module_manager
                    .get_by_module_id(input.module_id())
                    .cloned(),
            ))
            .await;
    }

    async fn on_server_module_load_message(&mut self, message: ServerModuleLoadMessage) {
        let (input, sender) = message.into();
        let (module,) = input.into();

        sender
            .do_send(ServerModuleLoadMessageOutput::new(
                self.module_manager.load(module),
            ))
            .await;
    }

    async fn on_server_module_unload_message(&mut self, message: ServerModuleUnloadMessage) {
        let (input, sender) = message.into();

        sender
            .do_send(ServerModuleUnloadMessageOutput::new(
                self.module_manager.unload(input.module_id()),
            ))
            .await;
    }

    async fn on_server_client_registration(&mut self, message: ServerClientRegistrationMessage) {
        let (input,) = message.into();
        let (sender, receiver) = input.into();

        let client_id = self.server_client_message_senders.len();

        let client = ServerClientTask::new(
            ServerClientId::from(client_id),
            self.sender.clone(),
            sender,
            receiver,
            self.server.clone(),
        );

        self.server_client_message_senders
            .push(Some(client.sender().clone()));

        client.spawn();
    }

    async fn on_server_client_resignation(&mut self, message: ServerClientResignationMessage) {
        let (input,) = message.into();

        self.server_client_message_senders
            .get_mut(usize::from(input.server_client_id()))
            .take();
    }

    async fn on_server_server_socket_address_get_message(
        &mut self,
        message: ServerServerSocketAddressGetMessage,
    ) {
        let (input, sender) = message.into();

        let server_socket_address = match self
            .server_socket_addresses
            .get(usize::from(input.server_id()))
        {
            None | Some(None) => None,
            Some(Some(server_socket_address)) => Some(*server_socket_address),
        };

        sender
            .do_send(ServerServerSocketAddressGetMessageOutput::new(
                server_socket_address,
            ))
            .await;
    }

    async fn on_server_server_id_get_message(&mut self, message: ServerServerIdGetMessage) {
        let (_input, sender) = message.into();
        sender
            .do_send(ServerServerIdGetMessageOutput::new(self.server_id))
            .await;
    }

    async fn on_server_leader_server_id_get_message(
        &mut self,
        message: ServerLeaderServerIdGetMessage,
    ) {
        let (_input, sender) = message.into();
        sender
            .do_send(ServerLeaderServerIdGetMessageOutput::new(
                self.leader_server_id,
            ))
            .await;
    }

    async fn on_server_server_node_message_sender_get_message(
        &mut self,
        message: ServerServerNodeMessageSenderGetMessage,
    ) {
        let (input, sender) = message.into();

        let server_node_message_sender = match self
            .server_node_message_senders
            .get(usize::from(input.server_id()))
        {
            None | Some(None) => None,
            Some(Some(server_node_message_sender)) => Some(server_node_message_sender.clone()),
        };

        sender
            .do_send(ServerServerNodeMessageSenderGetMessageOutput::new(
                server_node_message_sender,
            ))
            .await;
    }

    async fn on_server_server_node_message_sender_get_all_message(
        &mut self,
        message: ServerServerNodeMessageSenderGetAllMessage,
    ) {
        let (_input, sender) = message.into();

        sender
            .do_send(ServerServerNodeMessageSenderGetAllMessageOutput::new(
                self.server_node_message_senders
                    .iter()
                    .enumerate()
                    .flat_map(|(server_id, sender)| match sender {
                        None => None,
                        Some(sender) => Some((ServerId::from(server_id), sender.clone())),
                    })
                    .collect(),
            ))
            .await;
    }

    async fn on_server_commit_message(&mut self, message: ServerCommitMessage) {
        let (input, output_sender) = message.into();
        let (log_id, data) = input.into();

        if self.server_id == self.leader_server_id {
            let log_entry_id = match self
                .replicate(log_id, vec![(data.into(), None)])
                .await
                .into_iter()
                .next()
            {
                Some(log_entry_id) => log_entry_id,
                None => return,
            };

            if let Some(output_sender) = output_sender {
                self.commit_messages
                    .entry(log_entry_id)
                    .or_insert(Vec::default())
                    .push(
                        ServerCommitCommitMessage::new(ServerCommitCommitMessageInput::new(
                            output_sender,
                        ))
                        .into(),
                    );
            }

            self.acknowledge(log_id, &vec![log_entry_id], self.server_id)
                .await;
        } else {
            let log_entry_issue_id = {
                let current = self.current_log_entry_issue_id;
                self.current_log_entry_issue_id =
                    LogEntryIssueId::from(usize::from(self.current_log_entry_issue_id) + 1);

                current
            };

            if let Some(output_sender) = output_sender {
                self.commit_issue_senders
                    .insert(log_entry_issue_id, output_sender);
            }

            let sender = self
                .server_node_message_senders
                .get(usize::from(self.leader_server_id))
                .expect("")
                .as_ref()
                .expect("");
            sender
                .do_send_asynchronous(ServerNodeLogEntriesCommitMessageInput::new(
                    log_id,
                    data,
                    log_entry_issue_id,
                ))
                .await;
        }
    }

    async fn on_server_commit_commit_message(&mut self, message: ServerCommitCommitMessage) {
        let (input,) = message.into();
        let (sender,) = input.into();

        sender.notify().await;
    }

    async fn on_server_server_socket_addresses_get_message(
        &mut self,
        message: ServerServerSocketAddressesGetMessage,
    ) {
        let (_, sender) = message.into();

        sender
            .do_send(ServerServerSocketAddressesGetMessageOutput::new(
                self.server_socket_addresses.clone(),
            ))
            .await;
    }

    async fn on_server_log_create_message(&mut self, message: ServerLogCreateMessage) {
        let (input, output_sender) = message.into();
        let (log_issuer,) = input.into();

        if self.server_id == self.leader_server_id {
            let log_id = LogId::from(self.log_manager.len());
            let log_entry_ids = self
                .replicate(
                    SERVER_SYSTEM_LOG_ID,
                    vec![(
                        serialize_to_bytes(&ServerSystemLogEntryData::from(
                            ServerSystemCreateLogLogEntryData::new(log_id, log_issuer),
                        ))
                        .expect("")
                        .freeze(),
                        None,
                    )],
                )
                .await;

            self.commit_messages.insert(
                *log_entry_ids.get(0).expect(""),
                vec![
                    ServerCommitLogCreateMessage::new(ServerCommitLogCreateMessageInput::new(
                        output_sender,
                        log_id,
                    ))
                    .into(),
                ],
            );

            self.acknowledge(SERVER_SYSTEM_LOG_ID, &log_entry_ids, self.server_id)
                .await;
        } else {
            panic!("This should not happen");
        }
    }

    async fn on_server_commit_log_create_message(&mut self, message: ServerCommitLogCreateMessage) {
        let (input,) = message.into();
        let (output_sender, log_id) = input.into();

        output_sender
            .do_send(ServerLogCreateMessageOutput::new(log_id))
            .await;
    }

    async fn on_server_client_get_message(&mut self, message: ServerClientGetMessage) {
        let (input, output_sender) = message.into();
        let (server_client_id,) = input.into();

        output_sender
            .do_send(ServerClientGetMessageOutput::new(
                match self
                    .server_client_message_senders
                    .get(usize::from(server_client_id))
                {
                    Some(Some(sender)) => Some(sender.clone()),
                    None | Some(None) => None,
                },
            ))
            .await;
    }

    async fn replicate(
        &mut self,
        log_id: LogId,
        log_entries_data: Vec<(LogEntryData, Option<LogEntryIssuer>)>,
    ) -> Vec<LogEntryId> {
        let log = self.log_manager.get_mut(log_id).expect("existing log id");

        match log.type_mut() {
            LogType::Leader(leader_log) => {
                let log_entry_ids = leader_log.append(
                    log_entries_data,
                    self.server_socket_addresses
                        .iter()
                        .enumerate()
                        .flat_map(|node| node.1.as_ref().map(|_| node.0))
                        .map(ServerId::from)
                        .collect(),
                );

                let log_entries = log_entry_ids
                    .iter()
                    .filter_map(|log_entry_id| leader_log.entries().get(usize::from(*log_entry_id)))
                    .cloned()
                    .collect::<Vec<_>>();

                for server_node_socket_sender in self.server_node_message_senders.iter().flatten() {
                    server_node_socket_sender
                        .do_send(ServerNodeReplicationMessage::new(
                            ServerNodeReplicationMessageInput::new(
                                log_id,
                                log_entries.clone(),
                                leader_log.last_committed_log_entry_id(),
                                leader_log.current_term(),
                            ),
                        ))
                        .await;
                }

                log_entry_ids
            }
            LogType::Follower(_follower) => {
                unimplemented!()
            }
        }
    }

    async fn acknowledge(
        &mut self,
        log_id: LogId,
        log_entry_ids: &Vec<LogEntryId>,
        server_id: ServerId,
    ) {
        let log = self.log_manager.get_mut(log_id).expect("existing log");
        let leader_log = match log.r#type_mut() {
            LogType::Leader(leader) => leader,
            LogType::Follower(_follower) => {
                panic!("Cannot acknowledge with follower log");
            }
        };

        let from_last_committed_log_entry_id =
            leader_log.last_committed_log_entry_id().map(usize::from);

        for log_entry_id in log_entry_ids {
            if let Err(error) = leader_log.acknowledge(*log_entry_id, server_id) {
                match error {
                    LogError::NotAcknowledgeable | LogError::AlreadyAcknowledged => {}
                    error => {
                        error!("{}", error);
                    }
                }

                continue;
            }

            debug!(
                "Log ({}) entry ({}) acknowledged by server ({})",
                log_id, log_entry_id, server_id
            );
        }

        let to_last_committed_log_entry_id =
            leader_log.last_committed_log_entry_id().map(usize::from);

        let committed_log_entry_id_range = match (
            from_last_committed_log_entry_id,
            to_last_committed_log_entry_id,
        ) {
            (None, Some(to)) => Some(to..(to + 1)),
            (Some(from), Some(to)) => Some((from + 1)..(to + 1)),
            _ => None,
        };

        if let Some(committed_log_entry_id_range) = committed_log_entry_id_range {
            let last_committed_log_entry_id = leader_log.last_committed_log_entry_id();
            let current_term = leader_log.current_term();

            self.on_commit_log_entries(log_id, committed_log_entry_id_range.map(LogEntryId::from))
                .await;

            for server_node_message_sender in self.server_node_message_senders.iter().flatten() {
                server_node_message_sender
                    .do_send(ServerNodeReplicationMessage::new(
                        ServerNodeReplicationMessageInput::new(
                            log_id,
                            Vec::default(),
                            last_committed_log_entry_id.map(LogEntryId::from),
                            current_term,
                        ),
                    ))
                    .await;
            }
        }
    }

    async fn on_commit_log_entries<T>(&mut self, log_id: LogId, log_entry_ids: T)
    where
        T: Iterator<Item = LogEntryId>,
    {
        for log_entry_id in log_entry_ids {
            self.on_commit_log_entry(log_id, log_entry_id).await;
        }
    }

    async fn on_commit_log_entry(&mut self, log_id: LogId, log_entry_id: LogEntryId) {
        debug!("Log ({}) entry ({}) committed", log_id, log_entry_id);

        if log_id == SERVER_SYSTEM_LOG_ID {
            let log_entry = match self
                .log_manager
                .get(log_id)
                .expect("server system log")
                .r#type()
            {
                LogType::Leader(leader) => leader.entries().get(usize::from(log_entry_id)),
                LogType::Follower(follower) => {
                    match follower.entries().get(usize::from(log_entry_id)) {
                        None | Some(None) => None,
                        Some(Some(log_entry)) => Some(log_entry),
                    }
                }
            };

            if let Some(log_entry) = log_entry {
                match deserialize_from_bytes(log_entry.data())
                    .expect("deserialized ServerSystemLogEntryData")
                    .0
                {
                    ServerSystemLogEntryData::AddServer(data) => {
                        self.on_commit_add_server_log_entry_data(data.clone()).await
                    }
                    ServerSystemLogEntryData::RemoveServer(_data) => {
                        if self.leader_server_id != self.server_id {
                            todo!();
                        }
                    }
                    ServerSystemLogEntryData::CreateLog(data) => {
                        let (log_id, log_issuer) = data.into();

                        self.log_manager.insert(Log::new(
                            log_id,
                            if self.server_id == self.leader_server_id {
                                LogLeaderType::default().into()
                            } else {
                                LogFollowerType::default().into()
                            },
                            log_issuer,
                        ));

                        self.on_commit_create_log_data(log_id).await;
                    }
                }
            }

            for message in self
                .commit_messages
                .remove(&log_entry_id)
                .unwrap_or_default()
            {
                self.on_server_message(message).await;
            }
        } else {
            for message in self
                .commit_messages
                .remove(&log_entry_id)
                .unwrap_or_default()
            {
                self.on_server_message(message).await;
            }
        }

        if let Some(issuer) = self
            .log_manager
            .get(log_id)
            .expect("")
            .entries()
            .get(log_entry_id)
            .expect("")
            .issuer()
        {
            match issuer {
                LogEntryIssuer::Server(issuer) => {
                    if issuer.server_id() == self.server_id {
                        if let Some(sender) = self.commit_issue_senders.remove(&issuer.issue_id()) {
                            sender.notify().await;
                        }
                    }
                }
            }
        }

        for (module_id, module) in self.module_manager.iter() {
            self.commit_sender
                .do_send(CommitTaskMessage {
                    module: module.clone(),
                    input: ServerModuleCommitEventInput::new(
                        self.server.clone(),
                        module_id,
                        log_id,
                        log_entry_id,
                    ),
                })
                .await;
        }
    }

    async fn on_commit_add_server_log_entry_data(&mut self, data: AddServerLogEntryData) {
        let server_id = usize::from(data.server_id());

        if server_id >= self.server_socket_addresses.len() {
            self.server_socket_addresses
                .resize_with(server_id + 1, || None);
        }

        if server_id >= self.server_node_message_senders.len() {
            self.server_node_message_senders
                .resize_with(server_id + 1, || None);
        }

        if let Some(server_socket_addresses) = self.server_socket_addresses.get_mut(server_id) {
            *server_socket_addresses = Some(data.server_socket_address());
        }

        if let Some(server_node_message_sender) = self.server_node_message_senders.get_mut(server_id) && server_node_message_sender.is_none() && data.server_id() != self.server_id {
            let task = ServerNodeTask::new(data.server_id(), self.sender.clone(), None, self.server.clone());
            *server_node_message_sender = Some(task.sender().clone());

            task.spawn();
        }
    }

    async fn on_commit_create_log_data(&mut self, log_id: LogId) {
        if let Some(Some(server_node_message_sender)) = self.server_node_message_senders.get_mut(usize::from(self.leader_server_id)) && self.leader_server_id != self.server_id {
            server_node_message_sender.do_send_asynchronous(
                ServerNodeLogAppendInitiateMessageInput::new(log_id),
            ).await;
        }
    }
}
