use crate::general::{
    GeneralMessage, GeneralRecoveryRequestMessage, GeneralRecoveryRequestMessageInput,
    GeneralRecoveryResponseMessageInput, GeneralRegistrationRequestMessage,
    GeneralRegistrationRequestMessageInput, GeneralRegistrationResponseMessageInput,
};
use crate::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::server::connection::ServerConnectionTask;
use crate::server::member::{
    ServerMemberMessage, ServerMemberReplicateMessage, ServerMemberReplicateMessageInput,
    ServerMemberTask,
};
use crate::server::{
    AddServerLogEntryData, FollowingLog, LeadingLog, LogEntryData, LogEntryId, NewServerError,
    ServerAcknowledgeLogEntriesMessage, ServerFollowerType, ServerId, ServerLeaderType,
    ServerMessage, ServerRecoveryMessage, ServerRecoveryMessageNotALeaderOutput,
    ServerRegistrationMessage, ServerRegistrationMessageNotALeaderOutput,
    ServerReplicateLogEntriesMessage, ServerReplicateLogEntriesMessageOutput,
    ServerSocketAcceptMessage, ServerSocketAcceptMessageInput, ServerType,
};
use async_recursion::async_recursion;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::{select, spawn};
use tracing::{error, info};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerTask {
    server_id: ServerId,
    server_members: Vec<Option<(SocketAddr, Option<MessageQueueSender<ServerMemberMessage>>)>>,
    r#type: ServerType,
    tcp_listener: TcpListener,
    sender: MessageQueueSender<ServerMessage>,
    receiver: MessageQueueReceiver<ServerMessage>,
    commit_messages: HashMap<LogEntryId, Vec<ServerMessage>>,
}

impl ServerTask {
    pub async fn new<S, T>(
        listener_address: S,
        follower_data: Option<(T, Option<ServerId>)>,
    ) -> Result<Self, NewServerError>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;
        let (queue_sender, queue_receiver) = message_queue();

        match follower_data {
            None => Ok(Self {
                server_id: 0,
                server_members: vec![Some((tcp_listener.local_addr()?, None))],
                r#type: ServerLeaderType::new(LeadingLog::default()).into(),
                tcp_listener,
                sender: queue_sender,
                receiver: queue_receiver,
                commit_messages: HashMap::default(),
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
                                MessageError::UnexpectedMessage(format!("{:?}", message)).into()
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
                        FollowingLog::new(term, Vec::default(), None),
                        socket_sender,
                        socket_receiver,
                    )
                    .into(),
                    server_members: Vec::default(),
                    tcp_listener,
                    sender: queue_sender,
                    receiver: queue_receiver,
                    commit_messages: HashMap::default(),
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
                                MessageError::UnexpectedMessage(format!("{:?}", message)).into()
                            )
                        }
                    }
                };

                info!(
                    "Recovered with leader {} and term {}",
                    leader_server_id, term
                );

                Ok(Self {
                    server_id,
                    r#type: ServerFollowerType::new(
                        leader_server_id,
                        FollowingLog::new(term, Vec::default(), None),
                        socket_sender,
                        socket_receiver,
                    )
                    .into(),
                    server_members: Vec::default(),
                    tcp_listener,
                    sender: queue_sender,
                    receiver: queue_receiver,
                    commit_messages: HashMap::default(),
                })
            }
        }
    }

    pub async fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        loop {
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
    }

    #[async_recursion]
    async fn on_server_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::SocketAccept(message) => {
                self.on_server_socket_accept_message(message).await
            }
            ServerMessage::Registration(message) => {
                self.on_server_registration_message(message).await
            }
            ServerMessage::Recovery(message) => self.on_server_recovery_message(message).await,
            ServerMessage::ReplicateLogEntries(message) => {
                self.on_server_replicate_log_entries_message(message).await
            }
            ServerMessage::AcknowledgeLogEntries(message) => {
                self.on_server_acknowledge_log_entries_message(message)
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
            ServerType::Leader(leader) => {
                let member_server_id = {
                    let mut iterator = self.server_members.iter();
                    let mut index = 0;

                    loop {
                        if iterator.next().is_none() {
                            break index;
                        }

                        index += 1
                    }
                };

                let task = ServerMemberTask::new(member_server_id, self.sender.clone(), None, None);
                let sender = task.sender().clone();
                task.spawn();

                if member_server_id >= self.server_members.len() {
                    self.server_members
                        .resize_with(member_server_id + 1, || None);
                }

                *self
                    .server_members
                    .get_mut(member_server_id)
                    .expect("valid entry") = Some((input.server_socket_address(), Some(sender)));

                self.replicate(
                    vec![AddServerLogEntryData::new(
                        member_server_id,
                        input.server_socket_address(),
                    ).into()],
                    |_| None,
                )
                .await;
            }
            ServerType::Follower(follower) => {
                let leader_server_socket_address =
                    match self.server_members.get(follower.leader_server_id()) {
                        Some(Some(member)) => member.0,
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

    async fn on_server_recovery_message(&mut self, message: ServerRecoveryMessage) {
        let (input, sender) = message.into();

        match &self.r#type {
            ServerType::Leader(leader) => {}
            ServerType::Follower(follower) => {
                let leader_server_socket_address =
                    match self.server_members.get(follower.leader_server_id()) {
                        Some(Some(member)) => member.0,
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

    async fn on_server_replicate_log_entries_message(
        &mut self,
        message: ServerReplicateLogEntriesMessage,
    ) {
        let (input, sender) = message.into();
        let (log_entries_data,) = input.into();

        sender
            .do_send(ServerReplicateLogEntriesMessageOutput::new(
                self.replicate(log_entries_data, |log_entry_ids| None).await,
            ))
            .await;
    }

    async fn on_server_acknowledge_log_entries_message(
        &mut self,
        message: ServerAcknowledgeLogEntriesMessage,
    ) {
        let (input,) = message.into();

        self.acknowledge(input.log_entry_ids(), input.server_id())
            .await;
    }

    async fn replicate<F>(
        &mut self,
        log_entries_data: Vec<LogEntryData>,
        commit_messages_builder: F,
    ) -> Vec<LogEntryId>
    where
        F: Fn(LogEntryId) -> Option<Vec<ServerMessage>>,
    {
        match &mut self.r#type {
            ServerType::Leader(ref mut leader) => {
                let log_entry_ids = leader.log_mut().append(log_entries_data);

                let log_entries = log_entry_ids
                    .iter()
                    .map(|log_entry_id| leader.log().entries().get(*log_entry_id))
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>();

                for server_member in &self.server_members {
                    if let Some(Some(sender)) = server_member.as_ref().map(|member| &member.1) {
                        sender
                            .do_send(ServerMemberReplicateMessage::new(
                                ServerMemberReplicateMessageInput::new(log_entries.clone()),
                            ))
                            .await;
                    }
                }

                /*
                   compiler bug:
                */
                /*for member_sender in self
                    .server_members
                    .iter()
                    .flatten()
                    .map(|member| &member.1)
                    .flatten()
                {
                    member_sender
                        .do_send(ServerMemberReplicateMessage::new(
                            ServerMemberReplicateMessageInput::new(vec![]),
                        ))
                        .await;
                }*/

                for log_entry_id in &log_entry_ids {
                    let messages = match commit_messages_builder(*log_entry_id) {
                        None => continue,
                        Some(messages) => messages,
                    };

                    /*self.commit_messages
                        .entry(*log_entry_id)
                        .or_insert_with(Vec::default)
                        .extend(messages);*/
                }

                self.acknowledge(&log_entry_ids, self.server_id).await;

                log_entry_ids
            }
            ServerType::Follower(follower) => {
                unimplemented!()
            }
        }
    }

    async fn acknowledge(&mut self, log_entry_ids: &Vec<LogEntryId>, server_id: ServerId) {
        match &mut self.r#type {
            ServerType::Leader(ref mut leader) => {
                let from_last_committed_log_entry_id = leader.log().last_committed_log_entry_id();

                for log_entry_id in log_entry_ids {
                    if let Err(error) = leader.log_mut().acknowledge(*log_entry_id, server_id) {
                        error!("{}", error);
                    }
                }

                let to_last_committed_log_entry_id = leader.log().last_committed_log_entry_id();

                let committed_log_entry_ids = match (
                    from_last_committed_log_entry_id,
                    to_last_committed_log_entry_id,
                ) {
                    (None, Some(to)) => Some(0..to),
                    (Some(from), Some(to)) => Some(from..to),
                    _ => None,
                };

                if let Some(committed_log_entry_ids) = committed_log_entry_ids {
                    for committed_log_entry_id in committed_log_entry_ids {
                        for message in self
                            .commit_messages
                            .remove(&committed_log_entry_id)
                            .unwrap_or_default()
                        {
                            self.on_server_message(message).await;
                        }
                    }
                }
            }
            ServerType::Follower(follower) => {
                unimplemented!()
            }
        }
    }
}
