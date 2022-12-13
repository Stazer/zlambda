pub mod client;
pub mod connection;
pub mod log;

////////////////////////////////////////////////////////////////////////////////////////////////////

use connection::FollowerConnection;
use log::FollowerLog;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tracing::{error, trace};
use zlambda_common::log::{LogEntryData, LogEntryId};
use zlambda_common::message::{
    ClusterMessageRegisterResponse, LeaderToRegisteredFollowerMessage,
    LeaderToRegisteredFollowerMessageStreamReader, LeaderToUnregisteredFollowerMessage,
    LeaderToUnregisteredFollowerMessageStreamReader, MessageStreamReader, MessageStreamWriter,
    RegisteredFollowerToLeaderMessage, RegisteredFollowerToLeaderMessageStreamWriter,
    UnregisteredFollowerToLeaderMessage, UnregisteredFollowerToLeaderMessageStreamWriter,
};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum FollowerMessage {
    ReadLeaderAddress { sender: oneshot::Sender<SocketAddr> },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Follower {
    id: NodeId,
    leader_id: NodeId,
    log: FollowerLog,
    term: Term,
    tcp_listener: TcpListener,
    reader: LeaderToRegisteredFollowerMessageStreamReader,
    writer: RegisteredFollowerToLeaderMessageStreamWriter,
    sender: mpsc::Sender<FollowerMessage>,
    receiver: mpsc::Receiver<FollowerMessage>,
    addresses: HashMap<NodeId, SocketAddr>,
}

impl Follower {
    pub async fn new<T>(
        tcp_listener: TcpListener,
        registration_address: T,
    ) -> Result<Self, Box<dyn Error>>
    where
        T: ToSocketAddrs,
    {
        let address = tcp_listener.local_addr()?;

        let (reader, writer) = TcpStream::connect(registration_address).await?.into_split();

        let (mut reader, mut writer) = (
            LeaderToUnregisteredFollowerMessageStreamReader::new(reader),
            UnregisteredFollowerToLeaderMessageStreamWriter::new(writer),
        );

        writer
            .write(UnregisteredFollowerToLeaderMessage::RegisterRequest { address })
            .await?;

        let (id, leader_id, addresses, term) = match reader.read().await? {
            None => return Err("Expected message".into()),
            Some(LeaderToUnregisteredFollowerMessage::RegisterResponse(
                ClusterMessageRegisterResponse::Ok {
                    id,
                    leader_id,
                    addresses,
                    term,
                },
            )) => (id, leader_id, addresses, term),
            Some(_) => {
                return Err("Expected request response".into());
            }
        };

        let (sender, receiver) = mpsc::channel(16);

        trace!("Registered");

        Ok(Self {
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
        })
    }

    pub async fn run(&mut self) {
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

                    FollowerConnection::spawn(
                        MessageStreamReader::new(reader),
                        MessageStreamWriter::new(writer),
                        self.sender.clone(),
                    );
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

    async fn on_leader_to_follower_message(&mut self, message: LeaderToRegisteredFollowerMessage) {
        match message {
            LeaderToRegisteredFollowerMessage::AppendEntriesRequest {
                term,
                last_committed_log_entry_id,
                log_entry_data,
            } => {
                self.append_entries(term, last_committed_log_entry_id, log_entry_data)
                    .await
            }
        };
    }

    async fn append_entries(
        &mut self,
        _term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
    ) {
        let log_entry_ids = log_entry_data
            .iter()
            .map(|log_entry_data| log_entry_data.id())
            .collect();

        let result = self
            .writer
            .write(RegisteredFollowerToLeaderMessage::AppendEntriesResponse { log_entry_ids })
            .await;

        if let Err(error) = result {
            todo!("Switch to candidate {} {} {}", error, self.id, self.term);
        }

        for log_entry_data in log_entry_data.into_iter() {
            self.log.append(log_entry_data);
        }

        if let Some(last_committed_log_entry_id) = last_committed_log_entry_id {
            self.log.commit(last_committed_log_entry_id)
        }
    }
}
