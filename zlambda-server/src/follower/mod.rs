pub mod client;
pub mod connection;
pub mod log;

////////////////////////////////////////////////////////////////////////////////////////////////////

use connection::FollowerConnectionBuilder;
use log::FollowerLog;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::{error, trace};
use zlambda_common::channel::{DoReceive, DoSend};
use zlambda_common::message::{
    FollowerToGuestMessage, FollowerToLeaderAppendEntriesResponseMessage, FollowerToLeaderMessage,
    FollowerToLeaderMessageStreamWriter, GuestToNodeMessage, GuestToNodeMessageStreamWriter,
    LeaderToFollowerAppendEntriesRequestMessage, LeaderToFollowerDispatchResponseMessage,
    LeaderToFollowerMessage, LeaderToFollowerMessageStreamReader, LeaderToGuestMessage, Message,
    MessageStreamReader, MessageStreamWriter,
};
use zlambda_common::module::ModuleId;
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct FollowerPingMessage {
    sender: oneshot::Sender<()>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct FollowerLeaderAddressMessage {
    sender: oneshot::Sender<SocketAddr>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct FollowerDispatchMessage {
    module_id: ModuleId,
    payload: Vec<u8>,
    node_id: Option<NodeId>,
    sender: oneshot::Sender<Result<Vec<u8>, String>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct FollowerHandle {
    sender: mpsc::Sender<FollowerMessage>,
}

impl FollowerHandle {
    fn new(sender: mpsc::Sender<FollowerMessage>) -> Self {
        Self { sender }
    }

    pub async fn leader_address(&self) -> SocketAddr {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(FollowerMessage::LeaderAddress(
                FollowerLeaderAddressMessage { sender },
            ))
            .await;

        receiver.do_receive().await
    }

    pub async fn ping(&self) {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(FollowerMessage::Ping(FollowerPingMessage { sender }))
            .await;

        receiver.do_receive().await
    }

    pub async fn dispatch(
        &self,
        module_id: ModuleId,
        payload: Vec<u8>,
        node_id: Option<NodeId>,
    ) -> Result<Vec<u8>, String> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(FollowerMessage::Dispatch(FollowerDispatchMessage {
                module_id,
                payload,
                node_id,
                sender,
            }))
            .await;

        receiver.do_receive().await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerBuilder {
    sender: mpsc::Sender<FollowerMessage>,
    receiver: mpsc::Receiver<FollowerMessage>,
}

impl FollowerBuilder {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(16);

        Self { sender, receiver }
    }

    pub fn handle(&self) -> FollowerHandle {
        FollowerHandle::new(self.sender.clone())
    }

    pub async fn task<T>(
        self,
        tcp_listener: TcpListener,
        registration_address: T,
        node_id: Option<NodeId>,
    ) -> Result<FollowerTask, Box<dyn Error>>
    where
        T: ToSocketAddrs,
    {
        FollowerTask::new(
            self.sender,
            self.receiver,
            tcp_listener,
            registration_address,
            node_id,
        )
        .await
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum FollowerMessage {
    Ping(FollowerPingMessage),
    LeaderAddress(FollowerLeaderAddressMessage),
    Dispatch(FollowerDispatchMessage),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerTask {
    id: NodeId,
    leader_id: NodeId,
    log: FollowerLog,
    term: Term,
    tcp_listener: TcpListener,
    reader: LeaderToFollowerMessageStreamReader,
    writer: FollowerToLeaderMessageStreamWriter,
    sender: mpsc::Sender<FollowerMessage>,
    receiver: mpsc::Receiver<FollowerMessage>,
    addresses: HashMap<NodeId, SocketAddr>,
}

impl FollowerTask {
    async fn new<T>(
        sender: mpsc::Sender<FollowerMessage>,
        receiver: mpsc::Receiver<FollowerMessage>,
        tcp_listener: TcpListener,
        registration_address: T,
        node_id: Option<NodeId>,
    ) -> Result<Self, Box<dyn Error>>
    where
        T: ToSocketAddrs,
    {
        match node_id {
            None => {
                Self::new_registration(sender, receiver, tcp_listener, registration_address).await
            }
            Some(node_id) => {
                Self::new_handshake(
                    sender,
                    receiver,
                    tcp_listener,
                    registration_address,
                    node_id,
                )
                .await
            }
        }
    }

    async fn new_registration<T>(
        sender: mpsc::Sender<FollowerMessage>,
        receiver: mpsc::Receiver<FollowerMessage>,
        tcp_listener: TcpListener,
        registration_address: T,
    ) -> Result<Self, Box<dyn Error>>
    where
        T: ToSocketAddrs,
    {
        let address = tcp_listener.local_addr()?;
        let mut socket = TcpStream::connect(registration_address).await?;

        loop {
            let (reader, writer) = socket.into_split();

            let (mut reader, mut writer) = (
                MessageStreamReader::new(reader),
                GuestToNodeMessageStreamWriter::new(writer),
            );

            writer
                .write(GuestToNodeMessage::RegisterRequest { address })
                .await?;

            let (id, leader_id, addresses, term) = match reader.read().await? {
                None => return Err("Expected message".into()),
                Some(Message::FollowerToGuest(
                    FollowerToGuestMessage::RegisterNotALeaderResponse { leader_address },
                )) => {
                    socket = TcpStream::connect(leader_address).await?;
                    continue;
                }
                Some(Message::LeaderToGuest(LeaderToGuestMessage::RegisterOkResponse {
                    id,
                    leader_id,
                    addresses,
                    term,
                })) => (id, leader_id, addresses, term),
                Some(_) => {
                    return Err("Expected request response".into());
                }
            };

            trace!("Registered");

            return Ok(Self {
                id,
                leader_id,
                term,
                tcp_listener,
                reader: reader.into(),
                writer: writer.into(),
                log: FollowerLog::default(),
                addresses,
                sender,
                receiver,
            });
        }
    }

    async fn new_handshake<T>(
        sender: mpsc::Sender<FollowerMessage>,
        receiver: mpsc::Receiver<FollowerMessage>,
        tcp_listener: TcpListener,
        registration_address: T,
        node_id: NodeId,
    ) -> Result<Self, Box<dyn Error>>
    where
        T: ToSocketAddrs,
    {
        let address = tcp_listener.local_addr()?;
        let mut socket = TcpStream::connect(registration_address).await?;

        loop {
            let (reader, writer) = socket.into_split();

            let (mut reader, mut writer) = (
                MessageStreamReader::new(reader),
                GuestToNodeMessageStreamWriter::new(writer),
            );

            writer
                .write(GuestToNodeMessage::HandshakeRequest { address, node_id })
                .await?;

            let leader_id = match reader.read().await? {
                None => return Err("Expected message".into()),
                Some(Message::FollowerToGuest(
                    FollowerToGuestMessage::HandshakeNotALeaderResponse { leader_address },
                )) => {
                    socket = TcpStream::connect(leader_address).await?;
                    continue;
                }
                Some(Message::LeaderToGuest(LeaderToGuestMessage::HandshakeErrorResponse {
                    message,
                })) => {
                    return Err(message.into());
                }
                Some(Message::LeaderToGuest(LeaderToGuestMessage::HandshakeOkResponse {
                    leader_id,
                })) => leader_id,
                Some(message) => {
                    return Err("Expected response".into());
                }
            };

            trace!("Handshaked");

            return Ok(Self {
                id: node_id,
                leader_id,
                term: 0,
                tcp_listener,
                reader: reader.into(),
                writer: writer.into(),
                log: FollowerLog::default(),
                addresses: HashMap::default(),
                sender,
                receiver,
            });
        }
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await;
        });
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

                    trace!("Connection {} created", address);

                    let (reader, writer) = stream.into_split();

                    FollowerConnectionBuilder::default().task(
                        MessageStreamReader::new(reader),
                        MessageStreamWriter::new(writer),
                        FollowerHandle::new(self.sender.clone()),
                    ).spawn();
                }
                read_result = self.reader.read() => {
                    let message = match read_result {
                        Err(error) => {
                            error!("{}", error);
                            break
                        }
                        Ok(None) => continue,
                        Ok(Some(message)) => message,
                    };

                    self.on_leader_to_follower_message(message).await;
                }
                receive_result = self.receiver.recv() => {
                    let message = match receive_result {
                        None => continue,
                        Some(message) => message,
                    };

                    self.on_follower_message(message).await
                }
            )
        }
    }

    async fn on_follower_message(&mut self, message: FollowerMessage) {
        match message {
            FollowerMessage::Ping(message) => self.on_follower_ping_message(message).await,
            FollowerMessage::LeaderAddress(message) => {
                self.on_follower_leader_address_message(message).await
            }
            FollowerMessage::Dispatch(message) => self.on_follower_dispatch_message(message).await,
        }
    }

    async fn on_leader_to_follower_message(&mut self, message: LeaderToFollowerMessage) {
        match message {
            LeaderToFollowerMessage::AppendEntriesRequest(message) => {
                self.on_leader_to_follower_append_entries_request_message(message)
                    .await
            }
            LeaderToFollowerMessage::DispatchResponse(message) => {
                self.on_leader_to_follower_dispatch_response_message(message)
                    .await
            }
        };
    }

    async fn on_leader_to_follower_append_entries_request_message(
        &mut self,
        message: LeaderToFollowerAppendEntriesRequestMessage,
    ) {
        let (term, last_committed_log_entry_id, log_entry_data) = message.into();

        let appended_log_entry_ids = log_entry_data
            .iter()
            .map(|log_entry_data| log_entry_data.id())
            .collect();

        for log_entry_data in log_entry_data.into_iter() {
            self.log.append(log_entry_data);
        }

        let missing_log_entry_ids = last_committed_log_entry_id
            .map(|last_committed_log_entry_id| self.log.commit(last_committed_log_entry_id, term))
            .unwrap_or_default();

        let result = self
            .writer
            .write(FollowerToLeaderMessage::AppendEntriesResponse(
                FollowerToLeaderAppendEntriesResponseMessage::new(
                    appended_log_entry_ids,
                    missing_log_entry_ids,
                ),
            ))
            .await;

        if let Err(error) = result {
            todo!("Switch to candidate {} {} {}", error, self.id, self.term);
        }
    }

    async fn on_leader_to_follower_dispatch_response_message(
        &mut self,
        message: LeaderToFollowerDispatchResponseMessage,
    ) {
        todo!()
    }

    async fn on_follower_ping_message(&mut self, message: FollowerPingMessage) {
        message.sender.do_send(()).await
    }

    async fn on_follower_leader_address_message(&mut self, message: FollowerLeaderAddressMessage) {
        message
            .sender
            .do_send(
                (*self
                    .addresses
                    .get(&self.leader_id)
                    .expect("Cannot find socket address of leader"))
                .clone(),
            )
            .await
    }

    async fn on_follower_dispatch_message(&mut self, message: FollowerDispatchMessage) {
        message.sender.do_send(Err("Unimplemented".into())).await
    }
}
