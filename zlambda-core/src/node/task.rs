use crate::channel::{DoReceive, DoSend};
use crate::message::{
    FollowerToGuestMessage, GuestToNodeMessage, GuestToNodeRecoveryRequestMessage,
    GuestToNodeRegisterRequestMessage, LeaderToGuestMessage, Message, MessageError,
    MessageStreamReader, MessageStreamWriter,
};
use crate::node::connection::NodeConnectionTask;
use crate::node::member::NodeMemberReference;
use crate::node::member::NodeMemberTask;
use crate::node::NodeId;
use crate::node::{
    CreateNodeError, NodeAction, NodeFollowerRegistrationMessage,
    NodeFollowerRegistrationNotALeaderError, NodeMessage,
    NodeReference, NodeSocketAcceptMessage,
};
use crate::term::Term;
use std::net::SocketAddr;
use tokio::net::ToSocketAddrs;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::{select, spawn};
use tracing::{error, info};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeTask {
    node_id: NodeId,
    leader_node_id: NodeId,
    node_addresses: Vec<SocketAddr>,
    member_references: Vec<NodeMemberReference>,
    term: Term,
    tcp_listener: TcpListener,
    sender: mpsc::Sender<NodeMessage>,
    receiver: mpsc::Receiver<NodeMessage>,
}

impl NodeTask {
    pub async fn new<S, T>(
        listener_address: S,
        follower_data: Option<(T, Option<NodeId>)>,
    ) -> Result<Self, CreateNodeError>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let tcp_listener = TcpListener::bind(listener_address).await?;
        let (sender, receiver) = mpsc::channel(16);

        match follower_data {
            None => Ok(Self {
                node_id: 0,
                leader_node_id: 0,
                node_addresses: vec![tcp_listener.local_addr()?],
                member_references: Vec::default(),
                term: 0,
                tcp_listener,
                sender,
                receiver,
            }),
            Some((registration_address, None)) => {
                let address = tcp_listener.local_addr()?;
                let mut socket = TcpStream::connect(registration_address).await?;

                let (node_id, leader_node_id, node_addresses, term) = loop {
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
                    node_addresses,
                    member_references: Vec::default(),
                    term,
                    tcp_listener,
                    sender,
                    receiver,
                })
            }
            Some((recovery_address, Some(node_id))) => {
                let address = tcp_listener.local_addr()?;
                let mut socket = TcpStream::connect(recovery_address).await?;

                let (node_id, leader_node_id, node_addresses, term) = loop {
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
                    node_addresses,
                    member_references: Vec::default(),
                    term,
                    tcp_listener,
                    sender,
                    receiver,
                })
            }
        }
    }

    pub fn reference(&self) -> NodeReference {
        NodeReference::new(self.sender.clone())
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
    }

    pub async fn run(mut self) {
        loop {
            match self.select().await {
                NodeAction::Continue => {}
                NodeAction::Stop => break,
                NodeAction::Error(error) => {
                    error!("{}", error);
                    break;
                }
            }
        }
    }

    async fn select(&mut self) -> NodeAction {
        select!(
            result = self.tcp_listener.accept() => {
                let message = match result {
                    Err(error) => return error.into(),
                    Ok((socket, socket_address)) => NodeSocketAcceptMessage::new(socket_address, socket).into(),
                };

                self.on_node_message(message).await
            }
            message = self.receiver.do_receive() => {
                self.on_node_message(message).await
            }
        )
    }

    async fn on_node_message(&mut self, message: NodeMessage) -> NodeAction {
        match message {
            NodeMessage::SocketAccept(message) => self.on_node_socket_accept_message(message).await,
            NodeMessage::FollowerRegistration(message) => {
                self.on_node_follower_registration_message(message).await
            }
            _ => NodeAction::Stop,
        }
    }

    async fn on_node_socket_accept_message(
        &mut self,
        message: NodeSocketAcceptMessage,
    ) -> NodeAction {
        let (socket_address, stream) = message.into();
        let (reader, writer) = stream.into_split();
        let (reader, writer) = (
            MessageStreamReader::new(reader),
            MessageStreamWriter::new(writer),
        );

        info!("Connection {} created", socket_address);

        NodeConnectionTask::new(reader, writer, NodeReference::new(self.sender.clone()));

        NodeAction::Continue
    }

    async fn on_node_follower_registration_message(
        &mut self,
        message: NodeFollowerRegistrationMessage,
    ) -> NodeAction {
        let (socket_address, reader, writer, sender) = message.into();

        if self.leader_node_id != self.node_id {
            let leader_address = match self.node_addresses.get(self.leader_node_id) {
                None => return NodeAction::Stop,
                Some(leader_address) => *leader_address,
            };

            sender
                .do_send(Err(NodeFollowerRegistrationNotALeaderError::new(
                    leader_address, reader, writer,
                )
                .into()))
                .await;
        } else {
            self.node_addresses.push(socket_address);

            let task = NodeMemberTask::new(
                self.node_addresses.len() - 1,
                NodeReference::new(self.sender.clone()),
                Some(reader),
                Some(writer),
            );

            task.spawn();

            sender
                .do_send(Ok(()))
                .await;
        }

        NodeAction::Continue
    }
}
