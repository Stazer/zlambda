use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::common::module::ModuleManager;
use crate::common::net::{TcpListener, TcpStream, ToSocketAddrs};
use crate::common::runtime::{select, spawn};
use crate::general::{
    GeneralMessage, GeneralRecoveryRequestMessage, GeneralRecoveryRequestMessageInput,
    GeneralRecoveryResponseMessageInput, GeneralRegistrationRequestMessage,
    GeneralRegistrationRequestMessageInput, GeneralRegistrationResponseMessageInput,
};
use crate::server::client::{ServerClientId, ServerClientMessage, ServerClientTask};
use crate::server::connection::ServerConnectionTask;
use crate::server::node::ServerNodeLogAppendResponseMessageInput;
use crate::server::{
    AddServerLogEntryData, FollowingLog, LeadingLog, LogEntryData, LogEntryId, LogError,
    NewServerError, Server, ServerClientRegistrationMessage, ServerClientResignationMessage,
    ServerCommitCommitMessage, ServerCommitCommitMessageInput, ServerCommitMessage,
    ServerCommitRegistrationMessage, ServerCommitRegistrationMessageInput, ServerFollowerType,
    ServerId, ServerLeaderServerIdGetMessage, ServerLeaderServerIdGetMessageOutput,
    ServerLeaderType, ServerLogAppendRequestMessage, ServerLogEntriesAcknowledgementMessage,
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
    ServerServerNodeMessageSenderGetMessage, ServerServerNodeMessageSenderGetMessageOutput,
    ServerServerSocketAddressGetMessage, ServerServerSocketAddressGetMessageOutput,
    ServerServerSocketAddressesGetMessage, ServerServerSocketAddressesGetMessageOutput,
    ServerSocketAcceptMessage, ServerSocketAcceptMessageInput, ServerType,
};
use async_recursion::async_recursion;
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error, info};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct ServerTask {
    server_id: ServerId,
    leader_server_id: ServerId,
    r#type: ServerType,
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

        let (queue_sender, queue_receiver) = message_queue();
        let mut module_manager = ModuleManager::default();

        for module in modules {
            module_manager.load(module)?;
        }

        let server = Server::new(queue_sender.clone());

        match follower_data {
            None => Ok(Self {
                server_id: ServerId::default(),
                leader_server_id: ServerId::default(),
                r#type: ServerLeaderType::new(LeadingLog::default()).into(),
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
                    r#type: ServerFollowerType::new(
                        leader_server_id,
                        FollowingLog::new(term, Vec::default(), LogEntryId::from(0)),
                    )
                    .into(),
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
                    r#type: ServerFollowerType::new(
                        leader_server_id,
                        FollowingLog::new(term, Vec::default(), LogEntryId::from(0)),
                    )
                    .into(),
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
        for module in self.module_manager.iter() {
            module
                .on_startup(ServerModuleStartupEventInput::new(Arc::new(Server::from(
                    self.sender.clone(),
                ))))
                .await;
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

        for module in self.module_manager.iter() {
            module
                .on_shutdown(ServerModuleShutdownEventInput::new(self.server.clone()))
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
                self.on_server_log_append_request(message).await
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
            ServerMessage::Commit(message) => self.on_server_commit_message(message).await,
            ServerMessage::CommitCommit(message) => {
                self.on_server_commit_commit_message(message).await
            }
            ServerMessage::ServerSocketAddressesGet(message) => {
                self.on_server_server_socket_addresses_get_message(message)
                    .await
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

        match &self.r#type {
            ServerType::Leader(_) => {
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

                /*if node_server_id >= self.server_socket_addresses.len() {
                    self.server_socket_addresses
                        .resize_with(node_server_id + 1, || None);
                }

                if node_server_id >= self.server_node_message_senders.len() {
                    self.server_node_message_senders
                        .resize_with(node_server_id + 1, || None);
                }

                let task = ServerNodeTask::new(node_server_id.into(), self.sender.clone(), None);
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
                }*/

                let log_entry_ids = self
                    .replicate(vec![AddServerLogEntryData::new(
                        node_server_id.into(),
                        input.server_socket_address(),
                    )
                    .into()])
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

                self.acknowledge(&log_entry_ids, self.server_id).await;
            }
            ServerType::Follower(follower) => {
                let leader_server_socket_address = match self
                    .server_socket_addresses
                    .get(usize::from(follower.leader_server_id()))
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
    }

    async fn on_server_commit_registration_message(
        &mut self,
        message: ServerCommitRegistrationMessage,
    ) {
        let (input, output_sender) = message.into();

        let leader = match &self.r#type {
            ServerType::Leader(leader) => leader,
            ServerType::Follower(_) => panic!("Server should be leader"),
        };

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
                leader.log().last_committed_log_entry_id(),
                leader.log().current_term(),
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
            None | Some(None) => {
                sender.do_send(ServerRecoveryMessageOutput::Unknown).await;
                return;
            }
            Some(Some(server_node_message_senders)) => {
                let log = match &self.r#type {
                    ServerType::Leader(leader) => leader.log(),
                    ServerType::Follower(_) => {
                        panic!();
                    }
                };

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
                self.replicate(log_entries_data).await,
            ))
            .await;
    }

    async fn on_server_log_entries_acknowledgement_message(
        &mut self,
        message: ServerLogEntriesAcknowledgementMessage,
    ) {
        let (input,) = message.into();

        self.acknowledge(input.log_entry_ids(), input.server_id())
            .await;
    }

    async fn on_server_log_entries_recovery_message(
        &mut self,
        message: ServerLogEntriesRecoveryMessage,
    ) {
        let leader = match &self.r#type {
            ServerType::Leader(leader) => leader,
            ServerType::Follower(_) => {
                panic!("Server must be leader");
            }
        };

        let (input,) = message.into();

        let log_entries = input
            .log_entry_ids()
            .iter()
            .filter_map(|log_entry_id| leader.log().entries().get(usize::from(*log_entry_id)))
            .chain(
                leader
                    .log()
                    .acknowledgeable_log_entries_for_server(input.server_id()),
            )
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
                    log_entries,
                    leader.log().last_committed_log_entry_id(),
                    leader.log().current_term(),
                ),
            ))
            .await;
    }

    async fn on_server_log_append_request(&mut self, message: ServerLogAppendRequestMessage) {
        let follower = match &mut self.r#type {
            ServerType::Leader(_) => {
                panic!("Cannot append entries to leader node");
            }
            ServerType::Follower(ref mut follower) => follower,
        };

        let (input,) = message.into();
        let (server_id, log_entries, last_committed_log_entry_id, log_current_term) = input.into();

        let mut log_entry_ids = Vec::with_capacity(log_entries.len());

        for log_entry in log_entries.into_iter() {
            log_entry_ids.push(log_entry.id());
            follower.log_mut().push(log_entry);
        }

        let (missing_log_entry_ids, committed_log_entry_id_range) =
            match last_committed_log_entry_id {
                Some(last_committed_log_entry_id) => {
                    let from_last_committed_log_entry_id = follower
                        .log()
                        .last_committed_log_entry_id()
                        .map(usize::from);

                    let missing_log_entry_ids = follower
                        .log_mut()
                        .commit(last_committed_log_entry_id, log_current_term);

                    let to_last_committed_log_entry_id = follower
                        .log()
                        .last_committed_log_entry_id()
                        .map(usize::from);

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
                        log_entry_ids,
                        missing_log_entry_ids,
                    ))
                    .await;
            }
        }

        if let Some(committed_log_entry_id_range) = committed_log_entry_id_range {
            self.on_commit_log_entries(committed_log_entry_id_range.map(LogEntryId::from))
                .await;
        }
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

    async fn on_server_commit_message(&mut self, message: ServerCommitMessage) {
        let (input, output_sender) = message.into();
        let (data,) = input.into();

        let log_entry_id = match self.replicate(vec![data.into()]).await.into_iter().next() {
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

    async fn replicate(&mut self, log_entries_data: Vec<LogEntryData>) -> Vec<LogEntryId> {
        match &mut self.r#type {
            ServerType::Leader(ref mut leader) => {
                let log_entry_ids = leader.log_mut().append(
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
                    .filter_map(|log_entry_id| {
                        leader.log().entries().get(usize::from(*log_entry_id))
                    })
                    .cloned()
                    .collect::<Vec<_>>();

                for server_node_socket_sender in self.server_node_message_senders.iter().flatten() {
                    server_node_socket_sender
                        .do_send(ServerNodeReplicationMessage::new(
                            ServerNodeReplicationMessageInput::new(
                                log_entries.clone(),
                                leader.log().last_committed_log_entry_id(),
                                leader.log().current_term(),
                            ),
                        ))
                        .await;
                }

                log_entry_ids
            }
            ServerType::Follower(_follower) => {
                unimplemented!()
            }
        }
    }

    async fn acknowledge(&mut self, log_entry_ids: &Vec<LogEntryId>, server_id: ServerId) {
        let leader = match &mut self.r#type {
            ServerType::Leader(leader) => leader,
            ServerType::Follower(_follower) => {
                panic!("Cannot acknowledge as follower");
            }
        };

        let from_last_committed_log_entry_id =
            leader.log().last_committed_log_entry_id().map(usize::from);

        for log_entry_id in log_entry_ids {
            if let Err(error) = leader.log_mut().acknowledge(*log_entry_id, server_id) {
                match error {
                    LogError::NotAcknowledgeable | LogError::AlreadyAcknowledged => {}
                    error => {
                        error!("{}", error);
                    }
                }

                continue;
            }

            debug!(
                "Log entry {} acknowledged by server {}",
                log_entry_id, server_id
            );
        }

        let to_last_committed_log_entry_id =
            leader.log().last_committed_log_entry_id().map(usize::from);

        let committed_log_entry_id_range = match (
            from_last_committed_log_entry_id,
            to_last_committed_log_entry_id,
        ) {
            (None, Some(to)) => Some(to..(to + 1)),
            (Some(from), Some(to)) => Some((from + 1)..(to + 1)),
            _ => None,
        };

        if let Some(committed_log_entry_id_range) = committed_log_entry_id_range {
            let last_committed_log_entry_id = leader.log_mut().last_committed_log_entry_id();
            let current_term = leader.log_mut().current_term();

            self.on_commit_log_entries(committed_log_entry_id_range.map(LogEntryId::from))
                .await;

            for server_node_message_sender in self.server_node_message_senders.iter().flatten() {
                server_node_message_sender
                    .do_send(ServerNodeReplicationMessage::new(
                        ServerNodeReplicationMessageInput::new(
                            Vec::default(),
                            last_committed_log_entry_id.map(LogEntryId::from),
                            current_term,
                        ),
                    ))
                    .await;
            }
        }
    }

    async fn on_commit_log_entries<T>(&mut self, log_entry_ids: T)
    where
        T: Iterator<Item = LogEntryId>,
    {
        for log_entry_id in log_entry_ids {
            self.on_commit_log_entry(log_entry_id).await;
        }
    }

    async fn on_commit_log_entry(&mut self, log_entry_id: LogEntryId) {
        for module in self.module_manager.iter().cloned() {
            let server = self.server.clone();

            spawn(async move {
                module
                    .on_commit(ServerModuleCommitEventInput::new(server, log_entry_id))
                    .await;
            });
        }

        debug!("Committed log entry {}", log_entry_id,);

        let log_entry = match &self.r#type {
            ServerType::Leader(leader) => leader.log().entries().get(usize::from(log_entry_id)),
            ServerType::Follower(follower) => {
                match follower.log().entries().get(usize::from(log_entry_id)) {
                    None | Some(None) => None,
                    Some(Some(log_entry)) => Some(log_entry),
                }
            }
        };

        if let Some(log_entry) = log_entry {
            match log_entry.data() {
                LogEntryData::AddServer(data) => {
                    self.on_commit_add_server_log_entry_data(data.clone()).await
                }
                LogEntryData::RemoveServer(_data) => {
                    if self.leader_server_id != self.server_id {
                        todo!();
                    }
                }
                LogEntryData::Data(_) => {}
            }
        }

        for message in self
            .commit_messages
            .remove(&log_entry_id)
            .unwrap_or_default()
        {
            self.on_server_message(message).await;
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
}
