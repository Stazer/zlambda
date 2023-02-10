use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::common::module::ModuleManager;
use crate::common::net::{TcpListener, TcpStream, ToSocketAddrs};
use crate::common::runtime::{select, spawn};
use crate::common::utility::Bytes;
use crate::general::{
    GeneralLogEntriesAppendRequestMessage, GeneralLogEntriesAppendResponseMessage,
    GeneralLogEntriesAppendResponseMessageInput, GeneralMessage, GeneralNotifyMessage,
    GeneralRecoveryRequestMessage, GeneralRecoveryRequestMessageInput,
    GeneralRecoveryResponseMessageInput, GeneralRegistrationRequestMessage,
    GeneralRegistrationRequestMessageInput, GeneralRegistrationResponseMessageInput,
};
use crate::server::client::{ServerClientId, ServerClientMessage, ServerClientTask};
use crate::server::connection::ServerConnectionTask;
use crate::server::{
    AddServerLogEntryData, FollowingLog, LeadingLog, LogEntryData, LogEntryId, LogError,
    NewServerError, ServerClientRegistrationMessage, ServerClientResignationMessage,
    ServerCommitRegistrationMessage, ServerCommitRegistrationMessageInput, ServerFollowerType,
    ServerHandle, ServerId, ServerLeaderGeneralMessageMessage, ServerLeaderType,
    ServerLogEntriesAcknowledgementMessage, ServerLogEntriesRecoveryMessage,
    ServerLogEntriesReplicationMessage, ServerLogEntriesReplicationMessageOutput, ServerMessage,
    ServerModule, ServerModuleCommitEventInput, ServerModuleGetMessage,
    ServerModuleGetMessageOutput, ServerModuleLoadMessage, ServerModuleLoadMessageOutput,
    ServerModuleNotifyEventInput, ServerModuleNotifyEventInputServerSource,
    ServerModuleShutdownEventInput, ServerModuleStartupEventInput, ServerModuleUnloadMessage,
    ServerModuleUnloadMessageOutput, ServerNotifyMessage, ServerRecoveryMessage,
    ServerRecoveryMessageNotALeaderOutput, ServerRecoveryMessageOutput,
    ServerRecoveryMessageSuccessOutput, ServerRegistrationMessage,
    ServerRegistrationMessageNotALeaderOutput, ServerRegistrationMessageSuccessOutput,
    ServerSocketAcceptMessage, ServerSocketAcceptMessageInput, ServerType,
};
use crate::server::{
    ServerNodeMessage, ServerNodeReplicationMessage, ServerNodeReplicationMessageInput,
    ServerNodeTask,
};
use async_recursion::async_recursion;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error, info};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerTask {
    server_id: ServerId,
    server_nodes: Vec<Option<(SocketAddr, Option<MessageQueueSender<ServerNodeMessage>>)>>,
    r#type: ServerType,
    tcp_listener: TcpListener,
    sender: MessageQueueSender<ServerMessage>,
    receiver: MessageQueueReceiver<ServerMessage>,
    commit_messages: HashMap<LogEntryId, Vec<ServerMessage>>,
    module_manager: ModuleManager<dyn ServerModule>,
    clients: Vec<Option<MessageQueueSender<ServerClientMessage>>>,
}

impl ServerTask {
    pub async fn new<S, T>(
        listener_address: S,
        follower_data: Option<(T, Option<ServerId>)>,
        modules: impl Iterator<Item = Box<dyn ServerModule>>,
    ) -> Result<Self, NewServerError>
    where
        S: ToSocketAddrs + Debug,
        T: ToSocketAddrs + Debug,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;
        let (queue_sender, queue_receiver) = message_queue();
        let mut module_manager = ModuleManager::default();

        for module in modules {
            module_manager.load(Arc::from(module))?;
        }

        match follower_data {
            None => Ok(Self {
                server_id: ServerId::default(),
                server_nodes: vec![Some((tcp_listener.local_addr()?, None))],
                r#type: ServerLeaderType::new(LeadingLog::default()).into(),
                tcp_listener,
                sender: queue_sender,
                receiver: queue_receiver,
                commit_messages: HashMap::default(),
                module_manager,
                clients: Vec::default(),
            }),
            Some((registration_address, None)) => {
                let address = tcp_listener.local_addr()?;
                let mut socket = TcpStream::connect(registration_address).await?;

                let (
                    server_id,
                    leader_server_id,
                    _server_socket_addresses,
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

                Ok(Self {
                    server_id,
                    r#type: ServerFollowerType::new(
                        leader_server_id,
                        FollowingLog::new(term, Vec::default(), 0),
                        socket_sender,
                        socket_receiver,
                    )
                    .into(),
                    server_nodes: Vec::default(),
                    tcp_listener,
                    sender: queue_sender,
                    receiver: queue_receiver,
                    commit_messages: HashMap::default(),
                    module_manager,
                    clients: Vec::default(),
                })
            }
            Some((recovery_address, Some(server_id))) => {
                let mut socket = TcpStream::connect(recovery_address).await?;

                let (
                    leader_server_id,
                    _server_socket_addresses,
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

                Ok(Self {
                    server_id,
                    r#type: ServerFollowerType::new(
                        leader_server_id,
                        FollowingLog::new(term, Vec::default(), 0),
                        socket_sender,
                        socket_receiver,
                    )
                    .into(),
                    server_nodes: Vec::default(),
                    tcp_listener,
                    sender: queue_sender,
                    receiver: queue_receiver,
                    commit_messages: HashMap::default(),
                    module_manager,
                    clients: Vec::default(),
                })
            }
        }
    }

    pub fn handle(&self) -> ServerHandle {
        ServerHandle::new(self.sender.clone())
    }

    pub async fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        for module in self.module_manager.iter() {
            module
                .on_startup(ServerModuleStartupEventInput::new(ServerHandle::new(
                    self.sender.clone(),
                )))
                .await;
        }

        loop {
            match &mut self.r#type {
                ServerType::Leader(_) => {
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
                ServerType::Follower(follower) => {
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
                            ).await
                        }
                        message = self.receiver.do_receive() => {
                            self.on_server_message(message).await
                        }
                        result = follower.receiver_mut().receive() => {
                            let message = match result {
                                Ok(Some(message)) => message,
                                Ok(None) => {
                                    unimplemented!("switch to candiate");
                                }
                                Err(error) => {
                                    error!("{}", error);
                                    unimplemented!("switch to candiate");
                                    //continue;
                                }
                            };

                            self.on_general_message(message).await
                        }
                    )
                }
            }
        }

        for module in self.module_manager.iter() {
            module
                .on_shutdown(ServerModuleShutdownEventInput::new(ServerHandle::new(
                    self.sender.clone(),
                )))
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
            ServerMessage::LeaderGeneralMessage(message) => {
                self.on_server_leader_general_message_message(message).await
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
            ServerMessage::ModuleGet(message) => self.on_server_module_get_message(message).await,
            ServerMessage::ModuleLoad(message) => self.on_server_module_load_message(message).await,
            ServerMessage::ModuleUnload(message) => {
                self.on_server_module_unload_message(message).await
            }
            ServerMessage::Notify(message) => self.on_server_notify_message(message).await,
            ServerMessage::ClientRegistration(message) => {
                self.on_server_client_registration(message).await
            }
            ServerMessage::ClientResignation(message) => {
                self.on_server_client_resignation(message).await
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

    async fn on_server_leader_general_message_message(
        &mut self,
        message: ServerLeaderGeneralMessageMessage,
    ) {
        let (input,) = message.into();
        let (message,) = input.into();

        match message {
            GeneralMessage::LogEntriesAppendRequest(message) => {
                self.on_general_log_entries_append_request_message(message)
                    .await
            }
            message => {
                error!(
                    "{}",
                    MessageError::UnexpectedMessage(format!("{message:?}"))
                );
            }
        }
    }

    async fn on_server_registration_message(&mut self, message: ServerRegistrationMessage) {
        let (input, output_sender) = message.into();

        match &self.r#type {
            ServerType::Leader(_) => {
                let node_server_id = {
                    let mut iterator = self.server_nodes.iter();
                    let mut index = 0;

                    loop {
                        if iterator.next().is_none() {
                            break index;
                        }

                        index += 1
                    }
                };

                let task = ServerNodeTask::new(node_server_id.into(), self.sender.clone(), None);
                let sender = task.sender().clone();
                task.spawn();

                let log_entry_ids = self
                    .replicate(vec![AddServerLogEntryData::new(
                        node_server_id.into(),
                        input.server_socket_address(),
                    )
                    .into()])
                    .await;

                if node_server_id >= self.server_nodes.len() {
                    self.server_nodes.resize_with(node_server_id + 1, || None);
                }

                *self
                    .server_nodes
                    .get_mut(node_server_id)
                    .expect("valid entry") = Some((input.server_socket_address(), Some(sender)));

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
                    .server_nodes
                    .get(usize::from(follower.leader_server_id()))
                {
                    Some(Some(node)) => node.0,
                    _ => {
                        panic!("Expected leader server socket address");
                    }
                };

                output_sender
                    .do_send(ServerRegistrationMessageNotALeaderOutput::new(
                        leader_server_socket_address,
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

        let node = match self.server_nodes.get(usize::from(input.node_server_id())) {
            Some(Some(node)) => node,
            None | Some(None) => panic!("Server node {} should exist", input.node_server_id()),
        };

        let node_sender = match &node.1 {
            Some(node_sender) => node_sender.clone(),
            None => panic!(
                "Server node {} should have assigned sender",
                input.node_server_id()
            ),
        };

        output_sender
            .do_send(ServerRegistrationMessageSuccessOutput::new(
                input.node_server_id(),
                self.server_id,
                self.server_nodes
                    .iter()
                    .map(|node| node.as_ref().map(|x| x.0))
                    .collect(),
                leader.log().last_committed_log_entry_id(),
                leader.log().current_term(),
                node_sender,
            ))
            .await;
    }

    async fn on_server_recovery_message(&mut self, message: ServerRecoveryMessage) {
        let (input, sender) = message.into();

        match &self.r#type {
            ServerType::Leader(leader) => {
                match self.server_nodes.get(usize::from(input.server_id())) {
                    None | Some(None) => {
                        sender.do_send(ServerRecoveryMessageOutput::Unknown).await;
                    }
                    Some(Some(node)) => match &node.1 {
                        None => sender.do_send(ServerRecoveryMessageOutput::Unknown).await,
                        Some(node_sender) => {
                            sender
                                .do_send(ServerRecoveryMessageSuccessOutput::new(
                                    self.server_id,
                                    self.server_nodes
                                        .iter()
                                        .map(|node| node.as_ref().map(|x| x.0))
                                        .collect(),
                                    leader.log().last_committed_log_entry_id(),
                                    leader.log().current_term(),
                                    node_sender.clone(),
                                ))
                                .await;
                        }
                    },
                };
            }
            ServerType::Follower(follower) => {
                let leader_server_socket_address = match self
                    .server_nodes
                    .get(usize::from(follower.leader_server_id()))
                {
                    Some(Some(node)) => node.0,
                    _ => {
                        panic!("Expected leader server socket address");
                    }
                };

                sender
                    .do_send(ServerRecoveryMessageNotALeaderOutput::new(
                        leader_server_socket_address,
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
            .filter_map(|log_entry_id| leader.log().entries().get(*log_entry_id))
            .chain(
                leader
                    .log()
                    .acknowledgeable_log_entries_for_server(input.server_id()),
            )
            .cloned()
            .collect::<Vec<_>>();

        let node_queue_sender = match self
            .server_nodes
            .get(usize::from(input.server_id()))
            .map(|entry| entry.as_ref().map(|node| &node.1))
            .flatten()
        {
            Some(Some(node_queue_sender)) => node_queue_sender,
            None | Some(None) => {
                panic!("Node does not exist");
            }
        };

        node_queue_sender
            .do_send(ServerNodeReplicationMessage::new(
                ServerNodeReplicationMessageInput::new(
                    log_entries,
                    leader.log().last_committed_log_entry_id(),
                    leader.log().current_term(),
                ),
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

    async fn on_server_notify_message(&mut self, message: ServerNotifyMessage) {
        let (input,) = message.into();
        let (module_id, source, body) = input.into();

        let module = match self.module_manager.get_by_module_id(module_id) {
            None => return,
            Some(module) => module,
        };

        let sender = self.sender.clone();
        let module = module.clone();

        spawn(async move {
            module
                .on_notify(ServerModuleNotifyEventInput::new(
                    ServerHandle::new(sender),
                    source.into(),
                    body,
                ))
                .await;
        });
    }

    async fn on_server_client_registration(&mut self, message: ServerClientRegistrationMessage) {
        let (input,) = message.into();
        let (sender, receiver) = input.into();

        let client_id = self.clients.len();

        let client = ServerClientTask::new(
            ServerClientId::from(client_id),
            self.sender.clone(),
            sender,
            receiver,
        );

        self.clients.push(Some(client.sender().clone()));

        client.spawn();
    }

    async fn on_server_client_resignation(&mut self, message: ServerClientResignationMessage) {
        let (input,) = message.into();

        self.clients
            .get_mut(usize::from(input.server_client_id()))
            .take();
    }

    async fn on_general_message(&mut self, message: GeneralMessage) {
        match message {
            GeneralMessage::LogEntriesAppendRequest(message) => {
                self.on_general_log_entries_append_request_message(message)
                    .await
            }
            GeneralMessage::Notify(message) => self.on_general_notify_message(message).await,
            message => {
                error!(
                    "{}",
                    MessageError::UnexpectedMessage(format!("{message:?}"))
                );
            }
        }
    }

    async fn on_general_log_entries_append_request_message(
        &mut self,
        message: GeneralLogEntriesAppendRequestMessage,
    ) {
        let follower = match &mut self.r#type {
            ServerType::Leader(_) => {
                panic!("Cannot append entries to leader node");
            }
            ServerType::Follower(ref mut follower) => follower,
        };

        let (input,) = message.into();
        let (log_entries, last_committed_log_entry_id, log_current_term) = input.into();

        let mut log_entry_ids = Vec::with_capacity(log_entries.len());

        for log_entry in log_entries.into_iter() {
            log_entry_ids.push(log_entry.id());
            follower.log_mut().push(log_entry);
        }

        let (missing_log_entry_ids, committed_log_entry_id_range) =
            match last_committed_log_entry_id {
                Some(last_committed_log_entry_id) => {
                    let from_last_committed_log_entry_id =
                        follower.log().last_committed_log_entry_id();

                    let missing_log_entry_ids = follower
                        .log_mut()
                        .commit(last_committed_log_entry_id, log_current_term);

                    let to_last_committed_log_entry_id =
                        follower.log().last_committed_log_entry_id();

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

        let result = if !log_entry_ids.is_empty() || !missing_log_entry_ids.is_empty() {
            follower
                .sender_mut()
                .send(GeneralLogEntriesAppendResponseMessage::new(
                    GeneralLogEntriesAppendResponseMessageInput::new(
                        log_entry_ids,
                        missing_log_entry_ids,
                    ),
                ))
                .await
        } else {
            Ok(())
        };

        if let Some(committed_log_entry_id_range) = committed_log_entry_id_range {
            for committed_log_entry_id in committed_log_entry_id_range.clone() {
                let log_entry = match follower
                    .log()
                    .entries()
                    .get(committed_log_entry_id)
                    .cloned()
                {
                    None | Some(None) => continue,
                    Some(Some(log_entry)) => log_entry,
                };

                for module in self.module_manager.iter().cloned() {
                    let sender = self.sender.clone();
                    let log_entry = log_entry.clone();

                    spawn(async move {
                        module
                            .on_commit(ServerModuleCommitEventInput::new(
                                ServerHandle::new(sender),
                                log_entry,
                            ))
                            .await;
                    });
                }

                debug!("Committed log entry {}", committed_log_entry_id);
            }

            for committed_log_entry_id in committed_log_entry_id_range.clone() {
                for message in self
                    .commit_messages
                    .remove(&committed_log_entry_id)
                    .unwrap_or_default()
                {
                    self.on_server_message(message).await;
                }
            }
        }

        if let Err(error) = result {
            error!("{}", error);
            unimplemented!("Switch to candidate");
        }
    }

    async fn on_general_notify_message(&mut self, message: GeneralNotifyMessage) {
        let follower = match &self.r#type {
            ServerType::Leader(_) => return,
            ServerType::Follower(follower) => follower,
        };

        let (input,) = message.into();
        let (module_id, body) = input.into();

        let module = match self.module_manager.get_by_module_id(module_id) {
            None => return,
            Some(module) => module,
        };

        let sender = self.sender.clone();
        let module = module.clone();
        let leader_server_id = follower.leader_server_id();

        spawn(async move {
            module
                .on_notify(ServerModuleNotifyEventInput::new(
                    ServerHandle::new(sender),
                    ServerModuleNotifyEventInputServerSource::new(leader_server_id).into(),
                    body,
                ))
                .await;
        });
    }

    async fn replicate(&mut self, log_entries_data: Vec<LogEntryData>) -> Vec<LogEntryId> {
        match &mut self.r#type {
            ServerType::Leader(ref mut leader) => {
                let log_entry_ids = leader.log_mut().append(
                    log_entries_data,
                    self.server_nodes
                        .iter()
                        .enumerate()
                        .flat_map(|node| node.1.as_ref().map(|_| node.0))
                        .map(ServerId::from)
                        .collect(),
                );

                let log_entries = log_entry_ids
                    .iter()
                    .filter_map(|log_entry_id| leader.log().entries().get(*log_entry_id))
                    .cloned()
                    .collect::<Vec<_>>();

                for server_node in &self.server_nodes {
                    if let Some(Some(sender)) = server_node.as_ref().map(|node| &node.1) {
                        sender
                            .do_send(ServerNodeReplicationMessage::new(
                                ServerNodeReplicationMessageInput::new(
                                    log_entries.clone(),
                                    leader.log().last_committed_log_entry_id(),
                                    leader.log().current_term(),
                                ),
                            ))
                            .await;
                    }
                }

                /*
                   causes compiler bug:
                */
                /*for node_sender in self
                    .server_nodes
                    .iter()
                    .flatten()
                    .map(|node| &node.1)
                    .flatten()
                {
                    node_sender
                        .do_send(ServerNodeReplicationMessage::new(
                            ServerNodeReplicationMessageInput::new(vec![]),
                        ))
                        .await;
                }*/

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

        let from_last_committed_log_entry_id = leader.log().last_committed_log_entry_id();

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

        let to_last_committed_log_entry_id = leader.log().last_committed_log_entry_id();

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

            for committed_log_entry_id in committed_log_entry_id_range.clone() {
                let log_entry = match leader.log().entries().get(committed_log_entry_id).cloned() {
                    Some(log_entry) => log_entry,
                    None => continue,
                };

                for module in self.module_manager.iter().cloned() {
                    let sender = self.sender.clone();
                    let log_entry = log_entry.clone();

                    spawn(async move {
                        module
                            .on_commit(ServerModuleCommitEventInput::new(
                                ServerHandle::new(sender),
                                log_entry,
                            ))
                            .await;
                    });
                }

                debug!(
                    "Committed log entry {} by acknowledgement of server {}",
                    committed_log_entry_id, server_id
                );
            }

            for committed_log_entry_id in committed_log_entry_id_range {
                for message in self
                    .commit_messages
                    .remove(&committed_log_entry_id)
                    .unwrap_or_default()
                {
                    self.on_server_message(message).await;
                }
            }

            for server_node in &mut self.server_nodes {
                if let Some(Some(sender)) = server_node.as_ref().map(|node| &node.1) {
                    sender
                        .do_send(ServerNodeReplicationMessage::new(
                            ServerNodeReplicationMessageInput::new(
                                Vec::default(),
                                last_committed_log_entry_id,
                                current_term,
                            ),
                        ))
                        .await;
                }
            }
        }
    }
}
