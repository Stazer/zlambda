mod client;
mod connection;
mod follower;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use client::*;
pub use connection::*;
pub use follower::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::algorithm::next_key;
use crate::cluster::{ClientId, ConnectionId, NodeActor, NodeId, TermId};
use crate::common::{TcpListenerActor, TcpListenerActorAcceptMessage, UpdateRecipientActorMessage};
use actix::dev::{MessageResponse, OneshotSender};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::collections::HashMap;
use std::iter::once;
use std::net::SocketAddr;
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DestroyConnectionActorMessage {
    connection_id: ConnectionId,
}

impl From<DestroyConnectionActorMessage> for (ConnectionId,) {
    fn from(message: DestroyConnectionActorMessage) -> Self {
        (message.connection_id,)
    }
}

impl Message for DestroyConnectionActorMessage {
    type Result = ();
}

impl DestroyConnectionActorMessage {
    pub fn new(connection_id: ConnectionId) -> Self {
        Self { connection_id }
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CreateFollowerActorMessageResponse {
    node_id: NodeId,
    leader_node_id: NodeId,
    term_id: TermId,
    node_socket_addresses: HashMap<NodeId, SocketAddr>,
}

impl From<CreateFollowerActorMessageResponse>
    for (NodeId, NodeId, TermId, HashMap<NodeId, SocketAddr>)
{
    fn from(result: CreateFollowerActorMessageResponse) -> Self {
        (
            result.node_id,
            result.leader_node_id,
            result.term_id,
            result.node_socket_addresses,
        )
    }
}

impl<A, M> MessageResponse<A, M> for CreateFollowerActorMessageResponse
where
    A: Actor,
    M: Message<Result = Self>,
{
    fn handle(self, _context: &mut A::Context, sender: Option<OneshotSender<M::Result>>) {
        if let Some(sender) = sender {
            sender
                .send(self)
                .expect("Cannot send FollowerUpgradeActorMessageResponse");
        }
    }
}

impl CreateFollowerActorMessageResponse {
    pub fn new(
        node_id: NodeId,
        leader_node_id: NodeId,
        term_id: TermId,
        node_socket_addresses: HashMap<NodeId, SocketAddr>,
    ) -> Self {
        Self {
            node_id,
            leader_node_id,
            term_id,
            node_socket_addresses,
        }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    pub fn leader_node_id(&self) -> NodeId {
        self.leader_node_id
    }

    pub fn term_id(&self) -> TermId {
        self.term_id
    }

    pub fn node_socket_addresses(&self) -> &HashMap<NodeId, SocketAddr> {
        &self.node_socket_addresses
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct CreateFollowerActorMessage {
    socket_address: SocketAddr,
    follower_actor_address: Addr<LeaderNodeFollowerActor>,
}

impl From<CreateFollowerActorMessage> for (SocketAddr, Addr<LeaderNodeFollowerActor>) {
    fn from(message: CreateFollowerActorMessage) -> Self {
        (message.socket_address, message.follower_actor_address)
    }
}

impl Message for CreateFollowerActorMessage {
    type Result = CreateFollowerActorMessageResponse;
}

impl CreateFollowerActorMessage {
    pub fn new(
        socket_address: SocketAddr,
        follower_actor_address: Addr<LeaderNodeFollowerActor>,
    ) -> Self {
        Self {
            socket_address,
            follower_actor_address,
        }
    }

    pub fn socket_address(&self) -> &SocketAddr {
        &self.socket_address
    }

    pub fn follower_actor_address(&self) -> &Addr<LeaderNodeFollowerActor> {
        &self.follower_actor_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct DestroyFollowerActorMessage {
    node_id: NodeId,
}

impl From<DestroyFollowerActorMessage> for (NodeId,) {
    fn from(message: DestroyFollowerActorMessage) -> Self {
        (message.node_id,)
    }
}

impl Message for DestroyFollowerActorMessage {
    type Result = ();
}

impl DestroyFollowerActorMessage {
    pub fn new(node_id: NodeId) -> Self {
        Self { node_id }
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeActor {
    node_actor_address: Addr<NodeActor>,
    tcp_listener_actor_address: Addr<TcpListenerActor>,
    follower_actor_addresses: HashMap<NodeId, Addr<LeaderNodeFollowerActor>>,
    client_actor_addresses: HashMap<ClientId, Addr<LeaderNodeClientActor>>,
    term_id: TermId,
    node_id: NodeId,
    node_socket_addresses: HashMap<NodeId, SocketAddr>,
}

impl Actor for LeaderNodeActor {
    type Context = Context<Self>;
}

impl Handler<CreateFollowerActorMessage> for LeaderNodeActor {
    type Result = <CreateFollowerActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: CreateFollowerActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let node_id = next_key(
            self.follower_actor_addresses
                .keys()
                .chain(once(&self.node_id)),
        );

        let (socket_address, follower_actor_address) = message.into();
        self.follower_actor_addresses
            .insert(node_id, follower_actor_address);
        self.node_socket_addresses.insert(node_id, socket_address);

        Self::Result::new(
            node_id,
            self.node_id,
            self.term_id,
            self.node_socket_addresses.clone(),
        )
    }
}

impl Handler<DestroyFollowerActorMessage> for LeaderNodeActor {
    type Result = <DestroyFollowerActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: DestroyFollowerActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (node_id,) = message.into();
        self.follower_actor_addresses.remove(&node_id);
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
        let (result,) = message.into();

        match result {
            Ok(tcp_stream) => {
                LeaderNodeConnectionActor::new(context.address(), tcp_stream);
            }
            Err(error) => {
                error!("{}", error);
                context.stop();
            }
        }
    }
}

impl LeaderNodeActor {
    pub fn new(
        node_actor_address: Addr<NodeActor>,
        tcp_listener_actor_address: Addr<TcpListenerActor>,
        term_id: TermId,
        node_id: NodeId,
        node_socket_addresses: HashMap<NodeId, SocketAddr>,
    ) -> Addr<LeaderNodeActor> {
        Self::create(move |context| {
            tcp_listener_actor_address.do_send(UpdateRecipientActorMessage::new(
                context.address().recipient(),
            ));

            Self {
                node_actor_address,
                tcp_listener_actor_address,
                follower_actor_addresses: HashMap::default(),
                client_actor_addresses: HashMap::default(),
                term_id,
                node_id,
                node_socket_addresses,
            }
        })
    }
}
