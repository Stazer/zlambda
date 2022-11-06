use crate::cluster::{FollowerNodeActor, NodeActor};
use crate::common::{TcpListenerActor, TcpStreamActor};
use actix::Addr;
use std::fmt::{self, Debug, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct RegisteredFollowerNodeActorActorAddresses {
    node: Addr<NodeActor>,
    follower_node: Addr<FollowerNodeActor>,
    tcp_listener: Addr<TcpListenerActor>,
    tcp_stream: Addr<TcpStreamActor>,
}

impl Debug for RegisteredFollowerNodeActorActorAddresses {
    fn fmt(&self, _formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl RegisteredFollowerNodeActorActorAddresses {
    pub fn new(
        node: Addr<NodeActor>,
        follower_node: Addr<FollowerNodeActor>,
        tcp_listener: Addr<TcpListenerActor>,
        tcp_stream: Addr<TcpStreamActor>,
    ) -> Self {
        Self {
            node,
            follower_node,
            tcp_listener,
            tcp_stream,
        }
    }

    pub fn node(&self) -> &Addr<NodeActor> {
        &self.node
    }

    pub fn follower_node(&self) -> &Addr<FollowerNodeActor> {
        &self.follower_node
    }

    pub fn tcp_listener(&self) -> &Addr<TcpListenerActor> {
        &self.tcp_listener
    }

    pub fn tcp_stream(&self) -> &Addr<TcpStreamActor> {
        &self.tcp_stream
    }
}
