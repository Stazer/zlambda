use crate::message::{MessageQueueSender, MessageQueueReceiver, message_queue};
use crate::log::Log;
use crate::node::{NodeId, NodeInternalMessage, NewNodeError};
use tokio::net::{TcpListener, ToSocketAddrs};
use tokio::{spawn, select};
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct NodeTask {
    node_id: NodeId,
    leader_node_id: NodeId,
    node_socket_addresses: Vec<Option<SocketAddr>>,
    log: Log,
    tcp_listener: TcpListener,
    sender: MessageQueueSender<NodeInternalMessage>,
    receiver: MessageQueueReceiver<NodeInternalMessage>,
}

impl NodeTask {
    pub async fn new<S, T> (
        listener_address: S,
        follower_data: Option<(T, Option<NodeId>)>,
    ) -> Result<Self, NewNodeError>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;
        let (sender, receiver) = message_queue();

        match follower_data {
            None => Ok(Self {
                node_id: 0,
                leader_node_id: 0,
                node_socket_addresses: vec![Some(tcp_listener.local_addr()?)],
                log: Log::default(),
                tcp_listener,
                sender,
                receiver,
            }),
            Some((registration_address, None)) => {
                todo!()
            }
            Some((registration_address, node_id)) => {
                todo!()
            }
        }
    }

    /*pub async fn new<S, T>(
        listener_address: S,
        follower_data: Option<(T, Option<NodeId>)>,
    ) -> Result<Self, NewNodeError>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;
        let (sender, receiver) = message_channel();

        match follower_data {
            None => Ok(Self {
                node_id: 0,
                leader_node_id: 0,
                node_socket_addresses: vec![tcp_listener.local_addr()?],
                term: 0,
                tcp_listener,
                sender,
                receiver,
            }),
            Some((registration_address, None)) => {
                let address = tcp_listener.local_addr()?;
                let mut socket = TcpStream::connect(registration_address).await?;

                let (node_id, leader_node_id, node_socket_addresses, term) = loop {
                    let (reader, writer) = socket.into_split();

                    let (mut reader, mut writer) = (
                        MessageStreamReader::new(reader),
                        MessageStreamWriter::new(writer),
                    );

                    writer
                        .write(
                            GuestToNodeMessage::RegisterRequest(
                                GuestToNodeRegisterRequestMessage::new(address),
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
                    "Registered as node {} at leader {} with term {}",
                    node_id, leader_node_id, term
                );

                Ok(Self {
                    node_id,
                    leader_node_id,
                    node_socket_addresses,
                    term,
                    tcp_listener,
                    sender,
                    receiver,
                })
            }
            Some((recovery_address, Some(node_id))) => {
                let address = tcp_listener.local_addr()?;
                let mut socket = TcpStream::connect(recovery_address).await?;

                let (node_id, leader_node_id, node_socket_addresses, term) = loop {
                    let (reader, writer) = socket.into_split();

                    let (mut reader, mut writer) = (
                        MessageStreamReader::new(reader),
                        MessageStreamWriter::new(writer),
                    );

                    writer
                        .write(
                            GuestToNodeMessage::RecoveryRequest(
                                GuestToNodeRecoveryRequestMessage::new(address, node_id),
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

                info!("Recovered with leader {} and term {}", leader_node_id, term);

                Ok(Self {
                    node_id,
                    leader_node_id,
                    node_socket_addresses,
                    term,
                    tcp_listener,
                    sender,
                    receiver,
                })
            }
        }
    }*/

    pub async fn spawn(self) {
        spawn(async move {
            self.run().await
        });
    }

    pub async fn run(self) {
        loop {

        }
    }
}
