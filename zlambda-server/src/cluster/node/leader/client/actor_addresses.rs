use crate::common::TcpStreamActor;
use actix::Addr;
use std::fmt::{self, Debug, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderNodeClientActorAddresses {
    tcp_stream: Addr<TcpStreamActor>,
}

impl Debug for LeaderNodeClientActorAddresses {
    fn fmt(&self, _formatter: &mut Formatter) -> Result<(), fmt::Error> {
        Ok(())
    }
}

impl LeaderNodeClientActorAddresses {
    pub fn new(tcp_stream: Addr<TcpStreamActor>) -> Self {
        Self { tcp_stream }
    }

    pub fn tcp_stream(&self) -> &Addr<TcpStreamActor> {
        &self.tcp_stream
    }
}
