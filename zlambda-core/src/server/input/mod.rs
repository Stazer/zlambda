use tokio::net::TcpStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerSocketAcceptMessageInput {
    stream: TcpStream,
}

impl From<ServerSocketAcceptMessageInput> for (TcpStream,) {
    fn from(input: ServerSocketAcceptMessageInput) -> Self {
        (input.stream,)
    }
}

impl ServerSocketAcceptMessageInput {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn stream(&self) -> &TcpStream {
        &self.stream
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerRegistrationAttemptMessageInput {}
