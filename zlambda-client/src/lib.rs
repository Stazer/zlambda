use std::error::Error;
use std::path::Path;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::io::ReaderStream;
use tokio::fs::File;
use zlambda_common::message::{ClientMessage, Message, MessageStreamReader, MessageStreamWriter};
use tokio_stream::StreamExt;

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

    pub async fn load_module(&mut self, path: &Path) -> Result<u64, Box<dyn Error>> {
        let file = File::open(path).await?;

        self.writer.write(&Message::Client(ClientMessage::InitializeRequest)).await?;

        let id = match self.reader.read().await? {
            None => return Err("Expected response".into()),
            Some(Message::Client(ClientMessage::InitializeResponse { id })) => id,
            Some(_) => return Err("Expected response".into()),
        };

        let mut stream = ReaderStream::new(file);

        while let Some(bytes) = stream.next().await {
            let bytes = bytes?.to_vec();

            self.writer.write(&Message::Client(ClientMessage::AppendChunk {
                id,
                bytes,
            })).await?;
        }

        self.writer.write(&Message::Client(ClientMessage::LoadRequest { id })).await?;

        let id = match self.reader.read().await? {
            None => return Err("Expected response".into()),
            Some(Message::Client(ClientMessage::LoadResponse { id })) => id,
            Some(_) => return Err("Expected response".into()),
        };

        Ok(id)
    }

    pub async fn dispatch(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
