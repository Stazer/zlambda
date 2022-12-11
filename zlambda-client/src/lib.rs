use std::error::Error;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncRead;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;
use zlambda_common::dispatch::DispatchId;
use zlambda_common::message::{ClientMessage, Message, MessageStreamReader, MessageStreamWriter};
use zlambda_common::module::ModuleId;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Client {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    next_dispatch_id: DispatchId,
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

        Ok(Self {
            reader,
            writer,
            next_dispatch_id: 0,
        })
    }

    pub async fn load_module(&mut self, path: &Path) -> Result<u64, Box<dyn Error>> {
        let file = File::open(path).await?;

        self.writer
            .write(&Message::Client(ClientMessage::InitializeRequest))
            .await?;

        let id = match self.reader.read().await? {
            None => return Err("Expected response".into()),
            Some(Message::Client(ClientMessage::InitializeResponse(id))) => id,
            Some(_) => return Err("Expected response".into()),
        };

        let mut stream = ReaderStream::with_capacity(file, 4096 * 4);

        while let Some(bytes) = stream.next().await {
            let bytes = bytes?;

            if bytes.is_empty() {
                break;
            }

            self.writer
                .write(&Message::Client(ClientMessage::Append(id, bytes.to_vec())))
                .await?;
        }

        self.writer
            .write(&Message::Client(ClientMessage::LoadRequest(id)))
            .await?;

        let id = match self.reader.read().await? {
            None => return Err("Expected response".into()),
            Some(Message::Client(ClientMessage::LoadResponse(result))) => result?,
            Some(_) => return Err("Expected response".into()),
        };

        Ok(id)
    }

    pub async fn dispatch<T>(&mut self, id: ModuleId, reader: T) -> Result<Vec<u8>, Box<dyn Error>>
    where
        T: AsyncRead + Unpin,
    {
        let mut stream = ReaderStream::new(reader);
        let mut payload = Vec::new();

        while let Some(bytes) = stream.next().await {
            let bytes = bytes?;

            if bytes.is_empty() {
                break;
            }

            payload.extend(&bytes);
        }

        self.writer
            .write(&Message::Client(ClientMessage::DispatchRequest(
                id,
                self.next_dispatch_id,
                payload,
            )))
            .await?;

        self.next_dispatch_id += 1;

        let result = match self.reader.read().await? {
            None => return Err("Expected response".into()),
            Some(Message::Client(ClientMessage::DispatchResponse(_dispatch_id, payload))) => {
                payload
            }
            Some(_) => return Err("Expected response".into()),
        };

        let payload = result?;

        Ok(payload)
    }
}
