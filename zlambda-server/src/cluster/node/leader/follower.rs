use crate::cluster::{ConnectionId, NodeId, PacketReader};
use crate::common::{TcpStreamActor, TcpStreamActorReceiveMessage};
use actix::{Actor, Addr, Context, Handler, Message};

use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeFollowerActor {
    node_id: NodeId,
    connection_id: ConnectionId,
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    packet_reader: PacketReader,
}

impl Actor for LeaderNodeFollowerActor {
    type Context = Context<Self>;
}

impl Handler<TcpStreamActorReceiveMessage> for LeaderNodeFollowerActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (bytes,) = message.into();

        let bytes = match bytes {
            Ok(bytes) => bytes,
            Err(error) => {
                error!("{}", error);
                todo!();
            }
        };

        self.packet_reader.push(bytes);

        loop {
            let packet = match self.packet_reader.next() {
                Ok(Some(packet)) => packet,
                Ok(None) => {
                    break;
                }
                Err(error) => {
                    error!("{}", error);
                    todo!();
                }
            };

            match packet {
                _ => {
                    unimplemented!()
                }
            }
        }
    }
}

impl LeaderNodeFollowerActor {
    pub fn new(
        node_id: NodeId,
        connection_id: ConnectionId,
        tcp_stream_actor_address: Addr<TcpStreamActor>,
        packet_reader: PacketReader,
    ) -> Addr<Self> {
        (Self {
            node_id,
            connection_id,
            tcp_stream_actor_address,
            packet_reader,
        })
        .start()
    }
}
