pub mod connection;

////////////////////////////////////////////////////////////////////////////////////////////////////

/*use crate::algorithm::next_key;
use crate::cluster::node::{ClusterNodeId, ClusterNodeIncomingSender};
use crate::cluster::TermId;
use crate::tcp::{
    TcpListenerActor, TcpListenerIncomingSender, TcpListenerOutgoingReceiver, TcpStreamActor,
};
use connection::{ClusterLeaderNodeConnectionActor, ClusterLeaderNodeConnectionMessage};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::select;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ClusterLeaderNodeIncomingMessage {
    RegisterClusterNode {
        socket_address: SocketAddr,
        sender: oneshot::Sender<(
            ClusterNodeId,
            ClusterNodeId,
            HashMap<ClusterNodeId, SocketAddr>,
            TermId,
        )>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ClusterLeaderNodeIncomingSender = mpsc::Sender<ClusterLeaderNodeIncomingMessage>;
pub type ClusterLeaderNodeIncomingReceiver = mpsc::Receiver<ClusterLeaderNodeIncomingMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn cluster_leader_node_incoming_channel() -> (
    ClusterLeaderNodeIncomingSender,
    ClusterLeaderNodeIncomingReceiver,
) {
    mpsc::channel(16)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClusterLeaderNodeActor {
    cluster_node_id: ClusterNodeId,
    term_id: TermId,
    cluster_node_addresses: HashMap<ClusterNodeId, SocketAddr>,

    tcp_listener_incoming_sender: TcpListenerIncomingSender,
    tcp_listener_outgoing_receiver: TcpListenerOutgoingReceiver,
    cluster_leader_node_incoming_sender: ClusterLeaderNodeIncomingSender,
    cluster_node_incoming_sender: ClusterNodeIncomingSender,
}

impl ClusterLeaderNodeActor {
    pub async fn new<T>(address: T, cluster_node_incoming_sender: ClusterNodeIncomingSender) -> Result<ClusterLeaderNodeIncomingSender, Box<dyn Error>>
    where
        T: ToSocketAddrs,
    {
        let (tcp_listener_incoming_sender, tcp_listener_outgoing_receiver) =
            TcpListenerActor::new(TcpListener::bind(address).await?);

        let (cluster_leader_node_incoming_sender, cluster_leader_node_incoming_receiver) =
            cluster_leader_node_incoming_channel();

        let mut instance = Self {
            cluster_node_id: 0,
            term_id: 0,
            cluster_node_addresses: HashMap::default(),
            tcp_listener_incoming_sender,
            tcp_listener_outgoing_receiver,
            cluster_leader_node_incoming_sender: cluster_leader_node_incoming_sender.clone(),
            cluster_node_incoming_sender,
        };

        spawn(async move {
            instance.main().await;
        });

        Ok(cluster_leader_node_incoming_sender)
    }

    async fn main(&mut self) {
        loop {
            select!(
                /*message = self.receiver.cluster_leader_node_message.recv() => {
                    let message = match message {
                        None => break,
                        Some(message) => message,
                    };

                    match message {
                        ClusterLeaderNodeMessage::RegisterClusterNode { sender, socket_address } => {
                            let cluster_node_id = next_key(self.cluster_node_addresses.keys());
                            self.cluster_node_addresses.insert(cluster_node_id, socket_address);

                            let result = sender.send((
                                cluster_node_id,
                                self.cluster_node_id,
                                self.cluster_node_addresses.clone(),
                                self.term_id
                            ));

                            if result.is_err() {
                                self.cluster_node_addresses.remove(&cluster_node_id);
                                error!("Receiver dropped");
                            };
                        }
                    };
                },*/
                /*result = self.receiver.tcp_listener_result.recv() => {
                    let (stream, address) = match result {
                        None => break,
                        Some(Err(error)) => {
                            error!("{}", error);
                            break;
                        }
                        Some(Ok(data)) => data,
                    };

                    trace!("Connection from {}", address);

                    //let (tcp_stream_result_sender, tcp_stream_result_receiver) = mpsc::channel(16);

                    /*ClusterLeaderNodeConnectionActor::new(
                        tcp_stream_result_receiver,
                        TcpStreamActor::new(
                            tcp_stream_result_sender,
                            stream,
                        ),
                        self.sender.cluster_leader_node_message.clone(),
                    );*/
                }*/
            )
        }
    }
}*/
