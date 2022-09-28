use crate::cluster::{NodeClientId, Packet};
use actix::Message;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct PacketReaderActorReadPacketMessage {
    id: NodeClientId,
    packet: Packet,
}

impl From<PacketReaderActorReadPacketMessage> for (NodeClientId, Packet) {
    fn from(message: PacketReaderActorReadPacketMessage) -> Self {
        (message.id, message.packet)
    }
}

impl Message for PacketReaderActorReadPacketMessage {
    type Result = ();
}

impl PacketReaderActorReadPacketMessage {
    pub fn new(id: NodeClientId, packet: Packet) -> Self {
        Self { id, packet }
    }

    pub fn id(&self) -> NodeClientId {
        self.id
    }

    pub fn packet(&self) -> &Packet {
        &self.packet
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActorRemoveClientMessage {
    id: NodeClientId,
}

impl From<NodeActorRemoveClientMessage> for (NodeClientId,) {
    fn from(message: NodeActorRemoveClientMessage) -> Self {
        (message.id,)
    }
}

impl Message for NodeActorRemoveClientMessage {
    type Result = ();
}

impl NodeActorRemoveClientMessage {
    pub fn new(id: NodeClientId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> NodeClientId {
        self.id
    }
}
