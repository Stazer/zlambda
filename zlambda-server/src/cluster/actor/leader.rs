use crate::algorithm::next_key;
use crate::cluster::actor::{NodeActor, PacketReaderActor};
use crate::cluster::NodeId;
use crate::cluster::{
    ConnectionId, NodeActorRegisterMessage, NodeActorRemoveConnectionMessage,
    NodeRegisterResponsePacketError, Packet, PacketReaderActorReadPacketMessage, ReadPacketError,
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
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

type MessageId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct NodeNodeState {
    connection_id: Option<ConnectionId>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum ConnectionTypeState {
    IncomingRegisteringNode,
    OutgoingRegisteringNode,
    Node { node_id: NodeId },
    Client,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ConnectionNodeState {
    id: ConnectionId,
    r#type: Option<ConnectionTypeState>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct LeaderNodeNodeState {
    connection_id: ConnectionId,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct FollowerNodeNodeState {}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum NodeTypeState {
    Leader {
        node_id: NodeId,
        //nodes: HashMap<NodeId, NodeNodeState>,
    },
    Follower {
        node_id: NodeId,
        //nodes: HashMap<NodeId, NodeNodeState>
    },
    Candidate,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct NodeState {
    r#type: Option<NodeTypeState>,
    connections: HashMap<ConnectionId, ConnectionNodeState>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeConnection {
    connection_id: ConnectionId,
    stream_address: Addr<TcpStreamActor>,
    reader_address: Addr<PacketReaderActor>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeActor {
    node_address: Addr<NodeActor>,
    listener_address: Addr<TcpListenerActor>,
    connections: HashMap<ConnectionId, LeaderNodeConnection>,
}

impl Actor for LeaderNodeActor {
    type Context = Context<Self>;

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
                LeaderNodeConnection {
                    connection_id,
                    reader_address,
                    stream_address,
                }
            )
            .is_none());

        trace!("Connection {} created", connection_id);
    }
}

impl Handler<NodeActorRemoveConnectionMessage> for LeaderNodeActor {
    type Result = <NodeActorRemoveConnectionMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage,
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

impl Handler<PacketReaderActorReadPacketMessage> for LeaderNodeActor {
    type Result = <PacketReaderActorReadPacketMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (connection_id, packet): (ConnectionId, Packet) = message.into();

        let connection = self
            .connections
            .get(&connection_id)
            .expect("Connection not found");

        match packet {
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
            connections: HashMap::default(),
        })
    }
}
