use crate::cluster::{ConnectionId, LeaderNodeActor, NodeId, PacketReader, FollowerUpgradeActorMessage};
use crate::common::{TcpStreamActor, TcpStreamActorReceiveMessage};
use actix::{Actor, Addr, Context, Handler, Message, AsyncContext, MailboxError};
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeFollowerActor {
    leader_node_actor_address: Addr<LeaderNodeActor>,
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    node_id: NodeId,
    connection_id: ConnectionId,
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
    pub async fn new(
        leader_node_actor_address: Addr<LeaderNodeActor>,
        tcp_stream_actor_address: Addr<TcpStreamActor>,
        connection_id: ConnectionId,
        packet_reader: PacketReader,
    ) -> Result<Addr<Self>, MailboxError> {
        let context: <Self as Actor>::Context = Context::new();

        let node_id = leader_node_actor_address.send(FollowerUpgradeActorMessage::new(
            connection_id,
            context.address(),
        )).await?;

        Ok(context.run(Self {
            leader_node_actor_address,
            tcp_stream_actor_address,
            node_id,
            connection_id,
            packet_reader,
        }))
    }
}
