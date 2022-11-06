use crate::cluster::{ConnectionId, LeaderNodeClientActorAddresses, NodeId, PacketReader};
use crate::common::{TcpStreamActor, TcpStreamActorReceiveMessage};
use actix::{Actor, Addr, Context, Handler, Message};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeClientActor {
    actor_addresses: LeaderNodeClientActorAddresses,
    node_id: NodeId,
    connection_id: ConnectionId,
    packet_reader: PacketReader,
}

impl Actor for LeaderNodeClientActor {
    type Context = Context<Self>;
}

impl Handler<TcpStreamActorReceiveMessage> for LeaderNodeClientActor {
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

impl LeaderNodeClientActor {
    pub fn new(
        node_id: NodeId,
        connection_id: ConnectionId,
        tcp_stream_actor_address: Addr<TcpStreamActor>,
        packet_reader: PacketReader,
    ) -> Addr<Self> {
        (Self {
            actor_addresses: LeaderNodeClientActorAddresses::new(tcp_stream_actor_address),

            node_id,
            connection_id,
            packet_reader,
        })
        .start()
    }
}
