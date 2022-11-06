use crate::cluster::{ClientId, LeaderNodeClientActor, LeaderNodeFollowerActor, NodeActor, NodeId};
use crate::common::TcpListenerActor;
use actix::Addr;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderNodeActorActorAddresses {
    node: Addr<NodeActor>,
    tcp_listener: Addr<TcpListenerActor>,
    follower: HashMap<NodeId, Addr<LeaderNodeFollowerActor>>,
    clients: HashMap<ClientId, Addr<LeaderNodeClientActor>>,
}

impl Debug for LeaderNodeActorActorAddresses {
    fn fmt(&self, _formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl LeaderNodeActorActorAddresses {
    pub fn new(
        node: Addr<NodeActor>,
        tcp_listener: Addr<TcpListenerActor>,
        follower: HashMap<NodeId, Addr<LeaderNodeFollowerActor>>,
        clients: HashMap<ClientId, Addr<LeaderNodeClientActor>>,
    ) -> Self {
        Self {
            node,
            tcp_listener,
            follower,
            clients,
        }
    }

    pub fn node(&self) -> &Addr<NodeActor> {
        &self.node
    }

    pub fn tcp_listener(&self) -> &Addr<TcpListenerActor> {
        &self.tcp_listener
    }

    pub fn follower(&self) -> &HashMap<NodeId, Addr<LeaderNodeFollowerActor>> {
        &self.follower
    }

    pub fn follower_mut(&mut self) -> &mut HashMap<NodeId, Addr<LeaderNodeFollowerActor>> {
        &mut self.follower
    }

    pub fn clients(&self) -> &HashMap<ClientId, Addr<LeaderNodeClientActor>> {
        &self.clients
    }

    pub fn clients_mut(&mut self) -> &mut HashMap<ClientId, Addr<LeaderNodeClientActor>> {
        &mut self.clients
    }
}
