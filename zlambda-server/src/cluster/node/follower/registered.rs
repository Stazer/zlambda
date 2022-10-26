use crate::cluster::{
    FollowerNodeActor, LogEntry, LogEntryId, NodeActor, NodeId, Packet, PacketReader, TermId,
    UpdateFollowerNodeActorMessage,
};
use crate::common::{
    TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor, TcpStreamActorReceiveMessage,
    TcpStreamActorSendMessage, UpdateRecipientActorMessage,
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, WrapFuture,
};
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::net::SocketAddr;
use tracing::{error, trace};

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
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RegisteredFollowerNodeActor {
    actor_addresses: RegisteredFollowerNodeActorActorAddresses,

    tcp_listener_socket_local_address: SocketAddr,
    packet_reader: PacketReader,

    node_socket_addresses: HashMap<NodeId, SocketAddr>,
    term_id: TermId,
    node_id: NodeId,
    leader_node_id: NodeId,

    log_entries: HashMap<LogEntryId, LogEntry>,
}

impl Actor for RegisteredFollowerNodeActor {
    type Context = Context<Self>;
}

impl Handler<TcpStreamActorReceiveMessage> for RegisteredFollowerNodeActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (bytes,) = message.into();

        let bytes = match bytes {
            Ok(bytes) => bytes,
            Err(error) => {
                error!("{}", error);
                context.stop();

                return;
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
                    context.stop();

                    return;
                }
            };

            match packet {
                Packet::LogEntryRequest { log_entry } => {
                    let bytes = Packet::LogEntrySuccessResponse {
                        log_entry_id: log_entry.id(),
                    }
                    .to_bytes()
                    .expect("Cannot write LogEntrySuccessResponse");

                    self.log_entries.insert(log_entry.id(), log_entry);

                    let future = self
                        .actor_addresses
                        .tcp_stream
                        .send(TcpStreamActorSendMessage::new(bytes));

                    context.wait(async move { future.await }.into_actor(self).map(
                        |result, _actor, context| {
                            match result {
                                Err(e) => {
                                    error!("{}", e);
                                    context.stop();
                                }
                                Ok(_) => {}
                            };
                        },
                    ));
                }
                packet => {
                    error!("Unhandled packet {:?} from leader", packet);
                }
            }
        }
    }
}

impl Handler<TcpListenerActorAcceptMessage> for RegisteredFollowerNodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        todo!()
    }
}

impl RegisteredFollowerNodeActor {
    pub async fn new(
        node_actor_address: Addr<NodeActor>,
        follower_node_actor_address: Addr<FollowerNodeActor>,
        tcp_listener_actor_address: Addr<TcpListenerActor>,
        tcp_listener_socket_local_address: SocketAddr,
        tcp_stream_actor_address: Addr<TcpStreamActor>,
        packet_reader: PacketReader,
        term_id: TermId,
        node_id: NodeId,
        leader_node_id: NodeId,
        node_socket_addresses: HashMap<NodeId, SocketAddr>,
    ) -> Addr<Self> {
        let context = Context::new();

        tcp_listener_actor_address
            .send(UpdateRecipientActorMessage::new(
                context.address().recipient(),
            ))
            .await
            .expect("Cannot send UpdateRecipientActorMessage");

        tcp_stream_actor_address
            .send(UpdateRecipientActorMessage::new(
                context.address().recipient(),
            ))
            .await
            .expect("Cannot send UpdateRecipientActorMessage");

        let actor = context.run(Self {
            actor_addresses: RegisteredFollowerNodeActorActorAddresses::new(
                node_actor_address,
                follower_node_actor_address.clone(),
                tcp_listener_actor_address,
                tcp_stream_actor_address,
            ),
            tcp_listener_socket_local_address,
            packet_reader,
            term_id,
            node_id,
            leader_node_id,
            node_socket_addresses,
            log_entries: HashMap::default(),
        });

        follower_node_actor_address
            .send(UpdateFollowerNodeActorMessage::new(
                FollowerNodeActor::Registered(actor.clone()),
            ))
            .await
            .expect("Cannot send UpdateFollowerNodeActorMessage");

        actor
    }
}
