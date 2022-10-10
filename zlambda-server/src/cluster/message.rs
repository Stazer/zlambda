use crate::cluster::{ConnectionId, Packet};
use actix::Message;
use std::fmt::Debug;
use tokio::net::{TcpStream, ToSocketAddrs};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActorRemoveConnectionMessage<T>
where
    T: Debug + 'static,
{
    connection_id: T,
}

impl<T> From<NodeActorRemoveConnectionMessage<T>> for (T,)
where
    T: Debug,
{
    fn from(message: NodeActorRemoveConnectionMessage<T>) -> Self {
        (message.connection_id,)
    }
}

impl<T> Message for NodeActorRemoveConnectionMessage<T>
where
    T: Debug + 'static,
{
    type Result = ();
}

impl<T> NodeActorRemoveConnectionMessage<T>
where
    T: Debug,
{
    pub fn new(connection_id: T) -> Self {
        Self { connection_id }
    }

    pub fn connection_id(&self) -> &T {
        &self.connection_id
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
pub struct PacketReaderActorReadPacketMessage<T>
where
    T: Debug + 'static,
{
    id: T,
    packet: Packet,
}

impl<T> From<PacketReaderActorReadPacketMessage<T>> for (T, Packet)
where
    T: Debug + 'static,
{
    fn from(message: PacketReaderActorReadPacketMessage<T>) -> Self {
        (message.id, message.packet)
    }
}

impl<T> Message for PacketReaderActorReadPacketMessage<T>
where
    T: Debug + 'static,
{
    type Result = ();
}

impl<T> PacketReaderActorReadPacketMessage<T>
where
    T: Debug + 'static,
{
    pub fn new(id: T, packet: Packet) -> Self {
        Self { id, packet }
    }

    pub fn id(&self) -> &T {
        &self.id
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
