use crate::cluster::{
    FollowerNodeActor, NodeActor, NodeId, PacketReader, TermId, UpdateFollowerNodeActorMessage,
};
use crate::common::{
    TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor, TcpStreamActorReceiveMessage,
    UpdateRecipientActorMessage,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::collections::HashMap;
use std::net::SocketAddr;
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RegisteredFollowerNodeActor {
    node_actor_address: Addr<NodeActor>,
    follower_node_actor_address: Addr<FollowerNodeActor>,
    tcp_listener_actor_address: Addr<TcpListenerActor>,
    tcp_listener_socket_local_address: SocketAddr,
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    packet_reader: PacketReader,
    node_socket_addresses: HashMap<NodeId, SocketAddr>,
    term_id: TermId,
    node_id: NodeId,
    leader_node_id: NodeId,
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

            trace!("{:?}", packet);
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
            node_actor_address,
            follower_node_actor_address: follower_node_actor_address.clone(),
            tcp_listener_actor_address,
            tcp_listener_socket_local_address,
            tcp_stream_actor_address,
            packet_reader,
            term_id,
            node_id,
            leader_node_id,
            node_socket_addresses,
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
