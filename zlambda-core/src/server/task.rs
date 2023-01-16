use crate::general::{
    GeneralMessage, GeneralRegistrationRequestMessage, GeneralRegistrationRequestMessageInput,
    GeneralRegistrationResponseMessageInput, GeneralRecoveryRequestMessageInput, GeneralRecoveryRequestMessage,
    GeneralRecoveryResponseMessageInput
};
use crate::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::server::connection::ServerConnectionTask;
use crate::server::{Log, NewServerError, ServerId, ServerMessage, ServerSocketAcceptMessage};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::{select, spawn};
use tracing::info;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerTask {
    server_id: ServerId,
    leader_server_id: ServerId,
    server_socket_addresses: Vec<Option<SocketAddr>>,
    log: Log,
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
        let (sender, receiver) = message_queue();

        match follower_data {
            None => Ok(Self {
                server_id: 0,
                leader_server_id: 0,
                server_socket_addresses: vec![Some(tcp_listener.local_addr()?)],
                log: Log::default(),
                tcp_listener,
                sender,
                receiver,
            }),
            Some((registration_address, None)) => {
                let address = tcp_listener.local_addr()?;
                let mut socket = TcpStream::connect(registration_address).await?;

                let (server_id, leader_server_id, server_socket_addresses, term) = loop {
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
                                        TcpStream::connect(input.leader_socket_address()).await?;
                                    continue;
                                }
                                GeneralRegistrationResponseMessageInput::Success(input) => {
                                    break input.into()
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
                    leader_server_id,
                    server_socket_addresses,
                    log: Log::default(),
                    tcp_listener,
                    sender,
                    receiver,
                })
            }
            Some((recovery_address, Some(server_id))) => {
                let address = tcp_listener.local_addr()?;
                let mut socket = TcpStream::connect(recovery_address).await?;

                let (leader_server_id, server_socket_addresses, term) = loop {
                    let (reader, writer) = socket.into_split();

                    let (mut sender, mut receiver) = (
                        MessageSocketSender::<GeneralMessage>::new(writer),
                        MessageSocketReceiver::<GeneralMessage>::new(reader),
                    );

                    sender
                        .send(
                            GeneralRecoveryRequestMessage::new(
                                GeneralRecoveryRequestMessageInput::new(server_id),
                            ),
                        )
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
                                    return Err(NewServerError::IsOnline);
                                }
                                GeneralRecoveryResponseMessageInput::Success(input) => {
                                    break input.into()
                                }
                            }
                        }
                        Some(message) => {
                            return Err(MessageError::UnexpectedMessage(format!("{:?}", message)).into())
                        }
                    }
                };

                info!("Recovered with leader {} and term {}", leader_server_id, term);

                Ok(Self {
                    server_id,
                    leader_server_id,
                    server_socket_addresses,
                    log: Log::default(),
                    tcp_listener,
                    sender,
                    receiver,
                })
            }
        }
    }

    /*pub async fn new<S, T>(
        listener_address: S,
        follower_data: Option<(T, Option<ServerId>)>,
    ) -> Result<Self, NewServerError>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;
        let (sender, receiver) = message_channel();

        match follower_data {
            None => Ok(Self {
                server_id: 0,
                leader_server_id: 0,
                server_socket_addresses: vec![tcp_listener.local_addr()?],
                term: 0,
                tcp_listener,
                sender,
                receiver,
            }),
            Some((registration_address, None)) => {
                let address = tcp_listener.local_addr()?;
                let mut socket = TcpStream::connect(registration_address).await?;

                let (server_id, leader_server_id, server_socket_addresses, term) = loop {
                    let (reader, writer) = socket.into_split();

                    let (mut reader, mut writer) = (
                        MessageStreamReader::new(reader),
                        MessageStreamWriter::new(writer),
                    );

                    writer
                        .write(
                            GuestToServerMessage::RegisterRequest(
                                GuestToServerRegisterRequestMessage::new(address),
                            )
                            .into(),
                        )
                        .await?;

                    match reader.read().await? {
                        None => return Err(MessageError::ExpectedMessage.into()),
                        Some(Message::FollowerToGuest(
                            FollowerToGuestMessage::RegisterNotALeaderResponse(message),
                        )) => {
                            socket = TcpStream::connect(message.leader_address()).await?;
                            continue;
                        }
                        Some(Message::LeaderToGuest(LeaderToGuestMessage::RegisterOkResponse(
                            message,
                        ))) => break message.into(),
                        Some(message) => {
                            return Err(MessageError::UnexpectedMessage(message).into())
                        }
                    }
                };

                info!(
                    "Registered as server {} at leader {} with term {}",
                    server_id, leader_server_id, term
                );

                Ok(Self {
                    server_id,
                    leader_server_id,
                    server_socket_addresses,
                    term,
                    tcp_listener,
                    sender,
                    receiver,
                })
            }
            Some((recovery_address, Some(server_id))) => {
                let address = tcp_listener.local_addr()?;
                let mut socket = TcpStream::connect(recovery_address).await?;

                let (server_id, leader_server_id, server_socket_addresses, term) = loop {
                    let (reader, writer) = socket.into_split();

                    let (mut reader, mut writer) = (
                        MessageStreamReader::new(reader),
                        MessageStreamWriter::new(writer),
                    );

                    writer
                        .write(
                            GuestToServerMessage::RecoveryRequest(
                                GuestToServerRecoveryRequestMessage::new(address, server_id),
                            )
                            .into(),
                        )
                        .await?;

                    match reader.read().await? {
                        None => return Err(MessageError::ExpectedMessage.into()),
                        Some(Message::FollowerToGuest(
                            FollowerToGuestMessage::RecoveryNotALeaderResponse(message),
                        )) => {
                            socket = TcpStream::connect(message.leader_address()).await?;
                            continue;
                        }
                        Some(Message::LeaderToGuest(
                            LeaderToGuestMessage::RecoveryErrorResponse(message),
                        )) => {
                            let (error,) = message.into();
                            return Err(error.into());
                        }
                        Some(Message::LeaderToGuest(LeaderToGuestMessage::RecoveryOkResponse(
                            message,
                        ))) => break message.into(),
                        Some(message) => {
                            return Err(MessageError::UnexpectedMessage(message).into())
                        }
                    }
                };

                info!("Recovered with leader {} and term {}", leader_server_id, term);

                Ok(Self {
                    server_id,
                    leader_server_id,
                    server_socket_addresses,
                    term,
                    tcp_listener,
                    sender,
                    receiver,
                })
            }
        }
    }*/

    pub async fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        loop {
            select!(
                result = self.tcp_listener.accept() => {
                }
            )
        }
    }

    pub async fn on_server_message(&mut self, message: ServerMessage) {
        match message {
            ServerMessage::SocketAccept(message) => {
                self.on_server_socket_accept_message(message).await
            }
        }
    }

    pub async fn on_server_socket_accept_message(&mut self, message: ServerSocketAcceptMessage) {
        let (input,) = message.into();
        let (stream,) = input.into();

        let (reader, writer) = stream.into_split();

        ServerConnectionTask::new(
            self.sender.clone(),
            MessageSocketSender::new(writer),
            MessageSocketReceiver::new(reader),
        )
        .spawn()
    }
}
