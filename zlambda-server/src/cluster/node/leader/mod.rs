mod client;
mod connection;
mod follower;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use client::*;
pub use connection::*;
pub use follower::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::algorithm::next_key;
use crate::cluster::{ClientId, ConnectionId, NodeActor, NodeId};
use crate::common::{TcpListenerActor, TcpListenerActorAcceptMessage, UpdateRecipientActorMessage};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::collections::HashMap;
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct FollowerUpgradeActorMessage {
    connection_id: ConnectionId,
    follower_actor_address: Addr<LeaderNodeFollowerActor>,
}

impl From<FollowerUpgradeActorMessage> for (ConnectionId, Addr<LeaderNodeFollowerActor>) {
    fn from(message: FollowerUpgradeActorMessage) -> Self {
        (message.connection_id, message.follower_actor_address)
    }
}

impl Message for FollowerUpgradeActorMessage {
    type Result = NodeId;
}

impl FollowerUpgradeActorMessage {
    pub fn new(
        connection_id: ConnectionId,
        follower_actor_address: Addr<LeaderNodeFollowerActor>,
    ) -> Self {
        Self {
            connection_id,
            follower_actor_address,
        }
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }

    pub fn follower_actor_address(&self) -> &Addr<LeaderNodeFollowerActor> {
        &self.follower_actor_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientUpgradeActorMessage {
    connection_id: ConnectionId,
    client_actor_address: Addr<LeaderNodeClientActor>,
}

impl From<ClientUpgradeActorMessage> for (ConnectionId, Addr<LeaderNodeClientActor>) {
    fn from(message: ClientUpgradeActorMessage) -> Self {
        (message.connection_id, message.client_actor_address)
    }
}

impl Message for ClientUpgradeActorMessage {
    type Result = ClientId;
}

impl ClientUpgradeActorMessage {
    pub fn new(
        connection_id: ConnectionId,
        client_actor_address: Addr<LeaderNodeClientActor>,
    ) -> Self {
        Self {
            connection_id,
            client_actor_address,
        }
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }

    pub fn client_actor_address(&self) -> &Addr<LeaderNodeClientActor> {
        &self.client_actor_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeActor {
    node_actor_address: Addr<NodeActor>,
    tcp_listener_actor_address: Addr<TcpListenerActor>,
    connection_actor_addresses: HashMap<ConnectionId, Addr<LeaderNodeConnectionActor>>,
    follower_connection_actor_addresses: HashMap<ConnectionId, Addr<LeaderNodeFollowerActor>>,
    follower_actor_addresses: HashMap<NodeId, Addr<LeaderNodeFollowerActor>>,
    client_connection_actor_addresses: HashMap<ConnectionId, Addr<LeaderNodeFollowerActor>>,
    client_actor_addresses: HashMap<ClientId, Addr<LeaderNodeFollowerActor>>,
}

impl Actor for LeaderNodeActor {
    type Context = Context<Self>;
}

impl Handler<FollowerUpgradeActorMessage> for LeaderNodeActor {
    type Result = <FollowerUpgradeActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: FollowerUpgradeActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        0
    }
}

impl Handler<ClientUpgradeActorMessage> for LeaderNodeActor {
    type Result = <ClientUpgradeActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: ClientUpgradeActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        0
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
                trace!("{:?} connected", tcp_stream.peer_addr());

                let connection_id = next_key(&self.connection_actor_addresses);
                self.connection_actor_addresses.insert(
                    connection_id,
                    LeaderNodeConnectionActor::new(context.address(), tcp_stream, connection_id),
                );
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
    ) -> Addr<LeaderNodeActor> {
        Self::create(move |context| {
            tcp_listener_actor_address.do_send(UpdateRecipientActorMessage::new(
                context.address().recipient(),
            ));

            Self {
                node_actor_address,
                tcp_listener_actor_address,
                connection_actor_addresses: HashMap::default(),
                follower_connection_actor_addresses: HashMap::default(),
                follower_actor_addresses: HashMap::default(),
                client_connection_actor_addresses: HashMap::default(),
                client_actor_addresses: HashMap::default(),
            }
        })
    }
}
