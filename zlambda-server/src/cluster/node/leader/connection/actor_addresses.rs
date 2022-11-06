use crate::cluster::LeaderNodeActor;
use crate::common::TcpStreamActor;
use actix::Addr;
use std::fmt::{self, Debug, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderNodeConnectionActorAddresses {
    leader_node: Addr<LeaderNodeActor>,
    tcp_stream: Addr<TcpStreamActor>,
}

impl Debug for LeaderNodeConnectionActorAddresses {
    fn fmt(&self, _formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl LeaderNodeConnectionActorAddresses {
    pub fn new(leader_node: Addr<LeaderNodeActor>, tcp_stream: Addr<TcpStreamActor>) -> Self {
        Self {
            leader_node,
            tcp_stream,
        }
    }

    pub fn leader_node(&self) -> &Addr<LeaderNodeActor> {
        &self.leader_node
    }

    pub fn tcp_stream(&self) -> &Addr<TcpStreamActor> {
        &self.tcp_stream
    }
}
