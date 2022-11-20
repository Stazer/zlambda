use std::error::Error;
use tokio::net::TcpStream;
use tokio::net::ToSocketAddrs;
use zlambda_common::message::{ClientMessage, Message, MessageStreamReader, MessageStreamWriter};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Client {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
}

impl Client {
    pub async fn new<T>(address: T) -> Result<Self, Box<dyn Error>>
    where
        T: ToSocketAddrs,
    {
        let (reader, writer) = TcpStream::connect(address).await?.into_split();
        let (mut reader, mut writer) = (
            MessageStreamReader::new(reader),
            MessageStreamWriter::new(writer),
        );

        writer
            .write(&Message::Client(ClientMessage::RegisterRequest))
            .await?;

        match reader.read().await {
            Ok(Some(Message::Client(ClientMessage::RegisterResponse))) => {}
            Err(error) => return Err(error.into()),
            _ => return Err("Expected response".into()),
        };

        Ok(Self { reader, writer })
    }
}
