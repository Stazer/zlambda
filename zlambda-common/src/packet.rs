use crate::operation::{Operation, OperationRequest, OperationResponse};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct UdpPacket<T>
where
    T: Operation,
{
    operation: T,
}

impl From<UdpPacket<OperationRequest>> for OperationRequest {
    fn from(packet: UdpPacket<OperationRequest>) -> Self {
        packet.operation
    }
}

impl<T> From<UdpPacket<T>> for (T,)
where
    T: DeserializeOwned + Serialize + Operation + 'static,
{
    fn from(packet: UdpPacket<T>) -> Self {
        (packet.operation,)
    }
}

impl<T> UdpPacket<T>
where
    T: DeserializeOwned + Serialize + Operation + 'static,
{
    pub fn new(operation: T) -> Self {
        Self { operation }
    }

    pub fn operation(&self) -> &T {
        &self.operation
    }

    pub fn operation_mut(&mut self) -> &mut T {
        &mut self.operation
    }

    pub fn set_operation(&mut self, operation: T) {
        self.operation = operation
    }
}

pub type UdpOperationRequestPacket = UdpPacket<OperationRequest>;
pub type UdpOperationResponsePacket = UdpPacket<OperationResponse>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize)]
pub struct TcpPacket<T>
where
    T: Operation,
{
    operation: T,
}

impl From<TcpPacket<OperationRequest>> for OperationRequest {
    fn from(packet: TcpPacket<OperationRequest>) -> Self {
        packet.operation
    }
}

impl<T> From<TcpPacket<T>> for (T,)
where
    T: DeserializeOwned + Serialize + Operation + 'static,
{
    fn from(packet: TcpPacket<T>) -> Self {
        (packet.operation,)
    }
}

impl<T> TcpPacket<T>
where
    T: DeserializeOwned + Serialize + Operation + 'static,
{
    pub fn new(operation: T) -> Self {
        Self { operation }
    }

    pub fn operation(&self) -> &T {
        &self.operation
    }

    pub fn operation_mut(&mut self) -> &mut T {
        &mut self.operation
    }

    pub fn set_operation(&mut self, operation: T) {
        self.operation = operation
    }
}

pub type TcpOperationRequestPacket = TcpPacket<OperationRequest>;
pub type TcpOperationResponsePacket = TcpPacket<OperationResponse>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize)]
pub struct UnixPacket<T>
where
    T: Operation,
{
    operation: T,
}

impl From<UnixPacket<OperationRequest>> for OperationRequest {
    fn from(packet: UnixPacket<OperationRequest>) -> Self {
        packet.operation
    }
}

impl<T> From<UnixPacket<T>> for (T,)
where
    T: DeserializeOwned + Serialize + Operation + 'static,
{
    fn from(packet: UnixPacket<T>) -> Self {
        (packet.operation,)
    }
}

impl<T> UnixPacket<T>
where
    T: DeserializeOwned + Serialize + Operation + 'static,
{
    pub fn new(operation: T) -> Self {
        Self { operation }
    }

    pub fn operation(&self) -> &T {
        &self.operation
    }

    pub fn operation_mut(&mut self) -> &mut T {
        &mut self.operation
    }

    pub fn set_operation(&mut self, operation: T) {
        self.operation = operation
    }
}

pub type UnixOperationRequestPacket = UnixPacket<OperationRequest>;
pub type UnixOperationResponsePacket = UnixPacket<OperationResponse>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Deserialize, Serialize)]
pub struct StdinPacket<T>
where
    T: Operation,
{
    operation: T,
}

impl From<StdinPacket<OperationRequest>> for OperationRequest {
    fn from(packet: StdinPacket<OperationRequest>) -> Self {
        packet.operation
    }
}

impl<T> From<StdinPacket<T>> for (T,)
where
    T: DeserializeOwned + Serialize + Operation + 'static,
{
    fn from(packet: StdinPacket<T>) -> Self {
        (packet.operation,)
    }
}

impl<T> StdinPacket<T>
where
    T: DeserializeOwned + Serialize + Operation + 'static,
{
    pub fn new(operation: T) -> Self {
        Self { operation }
    }

    pub fn operation(&self) -> &T {
        &self.operation
    }

    pub fn operation_mut(&mut self) -> &mut T {
        &mut self.operation
    }

    pub fn set_operation(&mut self, operation: T) {
        self.operation = operation
    }
}

pub type StdinOperationRequestPacket = StdinPacket<OperationRequest>;
pub type StdinOperationResponsePacket = StdinPacket<OperationResponse>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Packet: DeserializeOwned + Serialize {}

impl<T> Packet for UdpPacket<T> where T: DeserializeOwned + Serialize + Operation + 'static {}

impl<T> Packet for TcpPacket<T> where T: DeserializeOwned + Serialize + Operation + 'static {}

impl<T> Packet for UnixPacket<T> where T: DeserializeOwned + Serialize + Operation + 'static {}

impl<T> Packet for StdinPacket<T> where T: DeserializeOwned + Serialize + Operation + 'static {}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait OperationRequestPacket: Packet + Into<OperationRequest> {}

impl OperationRequestPacket for UdpOperationRequestPacket {}

impl OperationRequestPacket for UnixOperationRequestPacket {}

impl OperationRequestPacket for TcpOperationRequestPacket {}

impl OperationRequestPacket for StdinOperationRequestPacket {}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait OperationResponsePacket: Packet {}

impl OperationResponsePacket for UdpOperationResponsePacket {}

impl OperationResponsePacket for UnixOperationResponsePacket {}

impl OperationResponsePacket for TcpOperationResponsePacket {}

impl OperationResponsePacket for StdinOperationResponsePacket {}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum ReadPacketError {
    PostcardError(postcard::Error),
}

impl Debug for ReadPacketError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::PostcardError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for ReadPacketError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::PostcardError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for ReadPacketError {}

impl From<postcard::Error> for ReadPacketError {
    fn from(error: postcard::Error) -> Self {
        Self::PostcardError(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub enum WritePacketError {
    PostcardError(postcard::Error),
}

impl Debug for WritePacketError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::PostcardError(error) => Debug::fmt(error, formatter),
        }
    }
}

impl Display for WritePacketError {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            Self::PostcardError(error) => Display::fmt(error, formatter),
        }
    }
}

impl error::Error for WritePacketError {}

impl From<postcard::Error> for WritePacketError {
    fn from(error: postcard::Error) -> Self {
        Self::PostcardError(error)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn from_bytes<T>(bytes: &[u8]) -> Result<T, ReadPacketError>
where
    T: Packet + 'static,
{
    Ok(postcard::from_bytes(bytes)?)
}

pub fn to_vec<T>(packet: &T) -> Result<Vec<u8>, WritePacketError>
where
    T: Packet + 'static,
{
    Ok(postcard::to_allocvec(packet)?)
}
