use crate::cluster::LeaderNodeActor;
use crate::common::TcpStreamActor;
use actix::Addr;
use std::fmt::{self, Debug, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderNodeFollowerActorAddresses {
    leader_node: Addr<LeaderNodeActor>,
    tcp_stream: Option<Addr<TcpStreamActor>>,
}

impl Debug for LeaderNodeFollowerActorAddresses {
    fn fmt(&self, _formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl LeaderNodeFollowerActorAddresses {
    pub fn new(
        leader_node: Addr<LeaderNodeActor>,
        tcp_stream: Option<Addr<TcpStreamActor>>,
    ) -> Self {
        Self {
            leader_node,
            tcp_stream,
        }
    }

    pub fn leader_node(&self) -> &Addr<LeaderNodeActor> {
        &self.leader_node
    }

    pub fn tcp_stream(&self) -> &Option<Addr<TcpStreamActor>> {
        &self.tcp_stream
    }

    pub fn set_tcp_stream(&mut self, tcp_stream: Option<Addr<TcpStreamActor>>) {
        self.tcp_stream = tcp_stream
    }
}
