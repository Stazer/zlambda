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
use crate::server::{
    FollowingLog, LeadingLog, NewServerError, ServerFollowerType, ServerId, ServerLeaderType,
    ServerMessage, ServerRecoveryMessage, ServerRecoveryMessageNotALeaderOutput,
    ServerRegistrationMessage, ServerRegistrationMessageNotALeaderOutput,
    ServerReplicateLogEntriesMessage, ServerReplicateLogEntriesMessageOutput,
    ServerSocketAcceptMessage, ServerSocketAcceptMessageInput, ServerType, ServerAcknowledgeLogEntriesMessage,
};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::{select, spawn};
use tracing::{error, info};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerTask {
    server_id: ServerId,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    r#type: ServerType,
    tcp_listener: TcpListener,
    sender: MessageQueueSender<ServerMessage>,
    receiver: MessageQueueReceiver<ServerMessage>,
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
                server_socket_addresses: vec![Some(tcp_listener.local_addr()?)],
                r#type: ServerLeaderType::new(LeadingLog::default()).into(),
                tcp_listener,
                sender: queue_sender,
                receiver: queue_receiver,
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
                    server_socket_addresses,
                    r#type: ServerFollowerType::new(
                        leader_server_id,
                        FollowingLog::new(term, Vec::default(), None),
                        socket_sender,
                        socket_receiver,
                    )
                    .into(),
                    tcp_listener,
                    sender: queue_sender,
                    receiver: queue_receiver,
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
                    server_socket_addresses,
                    r#type: ServerFollowerType::new(
                        leader_server_id,
                        FollowingLog::new(term, Vec::default(), None),
                        socket_sender,
                        socket_receiver,
                    )
                    .into(),
                    tcp_listener,
                    sender: queue_sender,
                    receiver: queue_receiver,
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
            )
        }
    }

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
                self.on_server_acknowledge_log_entries_message(message).await
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
        let (input, sender) = message.into();

        match &self.r#type {
            ServerType::Leader(leader) => {}
            ServerType::Follower(follower) => {
                let leader_server_socket_address = match self
                    .server_socket_addresses
                    .get(follower.leader_server_id())
                {
                    Some(Some(leader_server_socket_address)) => *leader_server_socket_address,
                    _ => {
                        panic!("Expected leader server socket address");
                    }
                };

                sender
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
                let leader_server_socket_address = match self
                    .server_socket_addresses
                    .get(follower.leader_server_id())
                {
                    Some(Some(leader_server_socket_address)) => *leader_server_socket_address,
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

        match &mut self.r#type {
            ServerType::Leader(ref mut leader) => {
                sender
                    .do_send(ServerReplicateLogEntriesMessageOutput::new(
                        leader.log_mut().append(log_entries_data),
                    ))
                    .await;
            }
            ServerType::Follower(follower) => {
                unimplemented!()
            }
        }
    }

    async fn on_server_acknowledge_log_entries_message(
        &mut self,
        message: ServerAcknowledgeLogEntriesMessage,
    ) {
        let (input,) = message.into();

        match &mut self.r#type {
            ServerType::Leader(ref mut leader) => {
                let last_committed_log_entry_id = leader.log().last_committed_log_entry_id();

                for log_entry_id in input.log_entry_ids() {
                    leader.log_mut().acknowledge(*log_entry_id, input.server_id());
                }

                let last_committed_log_entry_id = leader.log().last_committed_log_entry_id();
            }
            ServerType::Follower(follower) => {
                unimplemented!()
            }
        }
    }
}
