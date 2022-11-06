use crate::algorithm::next_key;
use crate::cluster::{LogEntryType, NodeActor, NodeId, TermId};
use crate::common::{TcpListenerActor, TcpListenerActorAcceptMessage, UpdateRecipientActorMessage};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::collections::HashMap;
use std::iter::once;
use std::net::SocketAddr;
use tracing::{error, trace};

use crate::cluster::AcknowledgeLogEntryActorMessage;
use crate::cluster::ReplicateLogEntryActorMessage;
use crate::cluster::{
    BeginLogEntryActorMessage, CreateFollowerActorMessage, DestroyFollowerActorMessage,
    LeaderNodeActorActorAddresses, LeaderNodeActorLogEntries, LeaderNodeConnectionActor,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderNodeActor {
    actor_addresses: LeaderNodeActorActorAddresses,

    term_id: TermId,
    node_id: NodeId,
    node_socket_addresses: HashMap<NodeId, SocketAddr>,

    log_entries: LeaderNodeActorLogEntries,
}

impl Actor for LeaderNodeActor {
    type Context = Context<Self>;
}

impl Handler<CreateFollowerActorMessage> for LeaderNodeActor {
    type Result = <CreateFollowerActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: CreateFollowerActorMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let node_id = next_key(
            self.actor_addresses
                .follower()
                .keys()
                .chain(once(&self.node_id)),
        );

        let (socket_address, follower_actor_address) = message.into();
        self.actor_addresses
            .follower_mut()
            .insert(node_id, follower_actor_address);
        self.node_socket_addresses.insert(node_id, socket_address);

        context.notify(BeginLogEntryActorMessage::new(LogEntryType::UpdateNodes(
            self.node_socket_addresses.clone(),
        )));

        Self::Result::new(
            node_id,
            self.node_id,
            self.term_id,
            self.node_socket_addresses.clone(),
        )
    }
}

impl Handler<DestroyFollowerActorMessage> for LeaderNodeActor {
    type Result = <DestroyFollowerActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: DestroyFollowerActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (node_id,) = message.into();
        self.actor_addresses.follower_mut().remove(&node_id);
    }
}

impl Handler<TcpListenerActorAcceptMessage> for LeaderNodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result,) = message.into();

        match result {
            Ok(tcp_stream) => {
                LeaderNodeConnectionActor::new(context.address(), tcp_stream);
            }
            Err(error) => {
                error!("{}", error);
                context.stop();
            }
        }
    }
}

impl Handler<BeginLogEntryActorMessage> for LeaderNodeActor {
    type Result = <BeginLogEntryActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: BeginLogEntryActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (r#type,) = message.into();

        let log_entry_id = self.log_entries.begin(
            r#type.clone(),
            self.node_socket_addresses.keys().copied().collect(),
        );

        for node_id in self
            .log_entries
            .get(log_entry_id)
            .expect("Log entry should be uncommitted")
            .remaining_acknowledging_nodes()
            .filter(|x| **x != self.node_id)
        {
            self.actor_addresses
                .follower()
                .get(node_id)
                .expect("Node id should have an follower actor address")
                .try_send(ReplicateLogEntryActorMessage::new(
                    log_entry_id,
                    r#type.clone(),
                    self.log_entries.last_committed_log_entry_id(),
                ))
                .expect("Sending ReplicateLogEntryActorMessage should be successful");
        }

        self.log_entries.acknowledge(log_entry_id, self.node_id);

        if self
            .log_entries
            .get(log_entry_id)
            .expect("Log entry should exist")
            .is_committed()
        {
            trace!("Replicated {} on one node", log_entry_id);
        }
    }
}

impl Handler<AcknowledgeLogEntryActorMessage> for LeaderNodeActor {
    type Result = <AcknowledgeLogEntryActorMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: AcknowledgeLogEntryActorMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        self.log_entries
            .acknowledge(message.log_entry_id(), message.node_id());

        if self
            .log_entries
            .get(message.log_entry_id())
            .expect("Log entry should exist")
            .is_committed()
        {
            trace!(
                "Acknowledged {} on node {}",
                message.log_entry_id(),
                message.node_id(),
            );
        }
    }
}

impl LeaderNodeActor {
    pub fn new(
        node_actor_address: Addr<NodeActor>,
        tcp_listener_actor_address: Addr<TcpListenerActor>,
        term_id: TermId,
        node_id: NodeId,
        node_socket_addresses: HashMap<NodeId, SocketAddr>,
    ) -> Addr<LeaderNodeActor> {
        Self::create(move |context| {
            tcp_listener_actor_address.do_send(UpdateRecipientActorMessage::new(
                context.address().recipient(),
            ));

            Self {
                actor_addresses: LeaderNodeActorActorAddresses::new(
                    node_actor_address,
                    tcp_listener_actor_address,
                    HashMap::default(),
                    HashMap::default(),
                ),

                term_id,
                node_id,
                node_socket_addresses,

                log_entries: LeaderNodeActorLogEntries::default(),
            }
        })
    }
}
