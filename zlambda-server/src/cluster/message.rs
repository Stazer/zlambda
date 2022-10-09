use crate::cluster::{ConnectionId, Packet};
use actix::Message;
use tokio::net::{TcpStream, ToSocketAddrs};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActorRemoveConnectionMessage {
    connection_id: ConnectionId,
}

impl From<NodeActorRemoveConnectionMessage> for (ConnectionId,) {
    fn from(message: NodeActorRemoveConnectionMessage) -> Self {
        (message.connection_id,)
    }
}

impl Message for NodeActorRemoveConnectionMessage {
    type Result = ();
}

impl NodeActorRemoveConnectionMessage {
    pub fn new(connection_id: ConnectionId) -> Self {
        Self { connection_id }
    }

    pub fn connection_id(&self) -> ConnectionId {
        self.connection_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActorRegisterMessage {
    stream: TcpStream,
}

impl From<NodeActorRegisterMessage> for (TcpStream,) {
    fn from(message: NodeActorRegisterMessage) -> Self {
        (message.stream,)
    }
}

impl Message for NodeActorRegisterMessage {
    type Result = ();
}

impl NodeActorRegisterMessage {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct PacketReaderActorReadPacketMessage {
    id: ConnectionId,
    packet: Packet,
}

impl From<PacketReaderActorReadPacketMessage> for (ConnectionId, Packet) {
    fn from(message: PacketReaderActorReadPacketMessage) -> Self {
        (message.id, message.packet)
    }
}

impl Message for PacketReaderActorReadPacketMessage {
    type Result = ();
}

impl PacketReaderActorReadPacketMessage {
    pub fn new(id: ConnectionId, packet: Packet) -> Self {
        Self { id, packet }
    }

    pub fn id(&self) -> ConnectionId {
        self.id
    }

    pub fn packet(&self) -> &Packet {
        &self.packet
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderConnectActorMessage<T>
where
    T: ToSocketAddrs + 'static,
{
    socket_address: T,
}

impl<T> From<LeaderConnectActorMessage<T>> for (T,)
where
    T: ToSocketAddrs + 'static,
{
    fn from(message: LeaderConnectActorMessage<T>) -> Self {
        (message.socket_address,)
    }
}

impl<T> Message for LeaderConnectActorMessage<T>
where
    T: ToSocketAddrs + 'static,
{
    type Result = ();
}

impl<T> LeaderConnectActorMessage<T>
where
    T: ToSocketAddrs + 'static,
{
    pub fn new(socket_address: T) -> Self {
        Self { socket_address }
    }

    pub fn socket_address(&self) -> &T {
        &self.socket_address
    }
}
