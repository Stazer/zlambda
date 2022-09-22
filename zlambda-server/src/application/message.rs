use actix::Message;
use std::io;
use std::path::PathBuf;
use tokio::net::ToSocketAddrs;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct CreateUdpSocketMessage<T>
where
    T: ToSocketAddrs + 'static,
{
    socket_address: T,
}

impl<T> From<CreateUdpSocketMessage<T>> for (T,)
where
    T: ToSocketAddrs + 'static,
{
    fn from(message: CreateUdpSocketMessage<T>) -> Self {
        (message.socket_address,)
    }
}

impl<T> Message for CreateUdpSocketMessage<T>
where
    T: ToSocketAddrs + 'static,
{
    type Result = Result<(), io::Error>;
}

impl<T> CreateUdpSocketMessage<T>
where
    T: ToSocketAddrs + 'static,
{
    pub fn new(socket_address: T) -> Self {
        Self { socket_address }
    }

    pub fn socket_address(&self) -> &T {
        &self.socket_address
    }

    pub fn socket_address_mut(&mut self) -> &mut T {
        &mut self.socket_address
    }

    pub fn set_socket_address(&mut self, socket_address: T) {
        self.socket_address = socket_address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct CreateUnixListenerMessage {
    path: PathBuf,
}

impl From<CreateUnixListenerMessage> for (PathBuf,) {
    fn from(message: CreateUnixListenerMessage) -> Self {
        (message.path,)
    }
}

impl Message for CreateUnixListenerMessage {
    type Result = Result<(), io::Error>;
}

impl CreateUnixListenerMessage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn path_mut(&mut self) -> &mut PathBuf {
        &mut self.path
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path
    }
}
