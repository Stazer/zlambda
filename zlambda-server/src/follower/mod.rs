use crate::log::following::FollowingLog;
use crate::state::State;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::select;
use tracing::{error, trace};
use zlambda_common::log::LogEntryData;
use zlambda_common::message::{
    ClusterMessage, ClusterMessageRegisterResponse, Message, MessageStreamReader,
    MessageStreamWriter,
};
use zlambda_common::node::NodeId;
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerNode {
    id: NodeId,
    leader_id: NodeId,
    log: FollowingLog,
    term: Term,
    tcp_listener: TcpListener,
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    addresses: HashMap<NodeId, SocketAddr>,
    state: State,
}

impl FollowerNode {
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
            MessageStreamReader::new(reader),
            MessageStreamWriter::new(writer),
        );

        writer
            .write(&Message::Cluster(ClusterMessage::RegisterRequest {
                address,
            }))
            .await?;

        let (id, leader_id, addresses, term) = match reader.read().await? {
            None => return Err("Expected message".into()),
            Some(Message::Cluster(ClusterMessage::RegisterResponse(
                ClusterMessageRegisterResponse::Ok {
                    id,
                    leader_id,
                    addresses,
                    term,
                },
            ))) => (id, leader_id, addresses, term),
            Some(_) => {
                return Err("Expected request response".into());
            }
        };

        trace!("Registered");

        Ok(Self {
            id,
            leader_id,
            term,
            tcp_listener,
            reader,
            writer,
            log: FollowingLog::default(),
            addresses,
            state: State::default(),
        })
    }

    pub async fn run(&mut self) {
        loop {
            select!(
                read_result = self.reader.read() => {
                    let message = match read_result {
                        Err(error) => {
                            error!("{}", error);
                            break
                        }
                        Ok(None) => continue,
                        Ok(Some(message)) => message,
                    };

                    match message {
                        Message::Cluster(ClusterMessage::AppendEntriesRequest { term, log_entry_data }) => self.append_entries(term, log_entry_data).await,
                        message => {
                            error!("Unexpected message {:?}", message);
                            break
                        }
                    };
                }
            )
        }
    }

    async fn append_entries(&mut self, _term: Term, log_entry_data: Vec<LogEntryData>) {
        let log_entry_ids = log_entry_data
            .iter()
            .map(|log_entry_data| log_entry_data.id())
            .collect();

        let result = self
            .writer
            .write(&Message::Cluster(ClusterMessage::AppendEntriesResponse {
                log_entry_ids,
            }))
            .await;

        if let Err(error) = result {
            todo!("Switch to candidate {}", error);
        }

        for log_entry_data in log_entry_data.into_iter() {
            self.log.append(log_entry_data);
        }
    }
}
