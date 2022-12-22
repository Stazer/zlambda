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
use zlambda_common::log::{LogEntryData, LogEntryId};
use zlambda_common::message::{
    FollowerToGuestMessage, FollowerToLeaderMessage, FollowerToLeaderMessageStreamWriter,
    GuestToNodeMessage, GuestToNodeMessageStreamWriter, LeaderToFollowerMessage,
    LeaderToFollowerMessageStreamReader, LeaderToGuestMessage, Message, MessageStreamReader,
    MessageStreamWriter,
};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;
use zlambda_common::channel::{DoReceive, DoSend};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
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
            .do_send(FollowerMessage::ReadLeaderAddress { sender })
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
    ReadLeaderAddress { sender: oneshot::Sender<SocketAddr> },
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
                    message
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

                    FollowerConnectionBuilder::new().task(
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

                    match message {
                        FollowerMessage::ReadLeaderAddress { sender } => {
                            let address = match self.addresses.get(&self.leader_id) {
                                Some(address) => address,
                                None => {
                                    error!("Cannot find address with node id {}", self.leader_id);
                                    break
                                }
                            };

                            if let Err(error) = sender.send(*address) {
                                error!("{}", error);
                                break
                            }
                        }
                    };
                }
            )
        }
    }

    async fn on_leader_to_follower_message(&mut self, message: LeaderToFollowerMessage) {
        match message {
            LeaderToFollowerMessage::AppendEntriesRequest {
                term,
                last_committed_log_entry_id,
                log_entry_data,
            } => {
                self.on_append_entries(term, last_committed_log_entry_id, log_entry_data)
                    .await
            }
        };
    }

    async fn on_append_entries(
        &mut self,
        _term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
    ) {
        let log_entry_ids = log_entry_data
            .iter()
            .map(|log_entry_data| log_entry_data.id())
            .collect();

        for log_entry_data in log_entry_data.into_iter() {
            self.log.append(log_entry_data);
        }

        if let Some(last_committed_log_entry_id) = last_committed_log_entry_id {
            self.log.commit(last_committed_log_entry_id)
        }

        let result = self
            .writer
            .write(FollowerToLeaderMessage::AppendEntriesResponse { log_entry_ids })
            .await;

        if let Err(error) = result {
            todo!("Switch to candidate {} {} {}", error, self.id, self.term);
        }
    }
}
