use crate::algorithm::next_key;
use crate::cluster::actor::{NodeActor, PacketReaderActor};
use crate::cluster::NodeId;
use crate::cluster::{
    ConnectionId, NodeActorRegisterMessage, NodeActorRemoveConnectionMessage,
    NodeRegisterResponsePacketError, Packet, PacketReaderActorReadPacketMessage, ReadPacketError,
    NodeRegisterResponsePacketSuccessData, NodeRegisterResponsePacketSuccessNodeData,
};
use crate::common::{
    ActorExecuteMessage, ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage,
    TcpStreamActor, TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, Recipient,
    ResponseActFuture, WrapFuture,
};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};
use std::time::Duration;

////////////////////////////////////////////////////////////////////////////////////////////////////

type MessageId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Node {
    id: NodeId,
    connection_id: ConnectionId,
    address: SocketAddr,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum ConnectionType {
    Client,
    Node(NodeId),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Connection {
    id: ConnectionId,
    r#type: Option<ConnectionType>,
    stream_address: Addr<TcpStreamActor>,
    reader_address: Addr<PacketReaderActor<ConnectionId>>,
    peer_address: SocketAddr,
    local_address: SocketAddr,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeActor {
    node_address: Addr<NodeActor>,
    listener_address: Addr<TcpListenerActor>,
    node_id: NodeId,
    nodes: HashMap<NodeId, Node>,
    connections: HashMap<ConnectionId, Connection>,
}

impl Actor for LeaderNodeActor {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        context.run_interval(Duration::new(1, 0), |actor, context| {
            for node in actor.nodes.values() {
                let connection = actor.connections.get(&node.connection_id).expect("Cannot find connection");
                connection.stream_address.do_send(TcpStreamActorSendMessage::new(
                    Packet::NodePing.to_bytes().expect("Cannot write NodePing")
                ));
            }
        });
    }

    fn stopped(&mut self, _context: &mut Self::Context) {
        self.node_address.do_send(ActorStopMessage);
        self.listener_address.do_send(ActorStopMessage);

        for connection in self.connections.values() {
            connection.stream_address.do_send(ActorStopMessage);
            connection.reader_address.do_send(ActorStopMessage);
        }
    }
}

impl Handler<ActorStopMessage> for LeaderNodeActor {
    type Result = <ActorStopMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(&mut self, _message: ActorStopMessage, context: &mut <Self as Actor>::Context) {
        context.stop();
    }
}

impl Handler<TcpListenerActorAcceptMessage> for LeaderNodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        trace!("Create connection");

        let stream = match message.into() {
            (Err(e),) => {
                error!("{}", e);
                return;
            }
            (Ok(s),) => s,
        };

        let connection_id = next_key(&self.connections);
        let peer_address = stream.peer_addr().expect("Cannot read peer address");
        let local_address = stream.local_addr().expect("Cannot read local address");

        let (reader_address, stream_address) = PacketReaderActor::new(
            connection_id,
            context.address().recipient(),
            context.address().recipient(),
            stream,
        );

        assert!(self
            .connections
            .insert(
                connection_id,
                Connection {
                    id: connection_id,
                    r#type: None,
                    reader_address,
                    stream_address,
                    peer_address,
                    local_address,
                },
            )
            .is_none());

        trace!("Connection {} created", connection_id);
    }
}

impl Handler<NodeActorRemoveConnectionMessage<ConnectionId>> for LeaderNodeActor {
    type Result = <NodeActorRemoveConnectionMessage<ConnectionId> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage<ConnectionId>,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        trace!("Destroy connection {}", message.connection_id());

        let connection = self
            .connections
            .get(&message.connection_id())
            .expect("Connection not found");

        connection.stream_address.do_send(ActorStopMessage);
        connection.reader_address.do_send(ActorStopMessage);

        assert!(self.connections.remove(&message.connection_id()).is_none());

        trace!("Connection destroyed");
    }
}

impl Handler<PacketReaderActorReadPacketMessage<ConnectionId>> for LeaderNodeActor {
    type Result = <PacketReaderActorReadPacketMessage<ConnectionId> as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage<ConnectionId>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (connection_id, packet): (ConnectionId, Packet) = message.into();

        let connection = self
            .connections
            .get_mut(&connection_id)
            .expect("Connection not found");

        match packet {
            Packet::NodeRegisterRequest { local_address } => {
                let node_id = next_key(&self.nodes);
                self.nodes.insert(
                    node_id,
                    Node {
                        id: node_id,
                        connection_id,
                        address: SocketAddr::new(connection.peer_address.ip(), local_address.port()),
                    }
                );
                connection.r#type = Some(ConnectionType::Node(node_id));

                connection.stream_address.do_send(TcpStreamActorSendMessage::new(
                    Packet::NodeRegisterResponse {
                        result: Ok(NodeRegisterResponsePacketSuccessData::new(
                            node_id,
                            self.nodes.values().map(|node| {
                                NodeRegisterResponsePacketSuccessNodeData::new(node.id, node.address.clone())
                            }).collect(),
                            self.node_id,
                        )),
                    }.to_bytes().expect("Cannot write NodeRegisterResponse")
                ));
            },
            _ => {
                error!(
                    "Unhandled packet {:?} by connection {}",
                    packet, connection_id
                );

                connection.stream_address.do_send(ActorStopMessage);
                connection.reader_address.do_send(ActorStopMessage);

                assert!(self.connections.remove(&connection_id).is_none());
            }
        }
    }
}

impl LeaderNodeActor {
    pub fn new(node_address: Addr<NodeActor>, listener: TcpListener) -> Addr<Self> {
        Self::create(|context| Self {
            node_address,
            listener_address: TcpListenerActor::new(
                context.address().recipient(),
                Some(context.address().recipient()),
                listener,
            ),
            node_id: 0,
            nodes: HashMap::default(),
            connections: HashMap::default(),
        })
    }
}
