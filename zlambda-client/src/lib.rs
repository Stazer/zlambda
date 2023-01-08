use futures::stream::StreamExt;
use std::error::Error;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncRead;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::io::ReaderStream;
use zlambda_common::dispatch::DispatchId;
use zlambda_common::message::{
    ClientToNodeAppendMessage, ClientToNodeDispatchRequestMessage,
    ClientToNodeInitializeRequestMessage, ClientToNodeLoadRequestMessage, ClientToNodeMessage,
    ClientToNodeMessageStreamWriter, NodeToClientMessage, NodeToClientMessageStreamReader,
};
use zlambda_common::module::ModuleId;
use zlambda_common::{Bytes, BytesMut};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct Client {
    reader: NodeToClientMessageStreamReader,
    writer: ClientToNodeMessageStreamWriter,
    next_dispatch_id: DispatchId,
}

impl Client {
    pub async fn new<T>(address: T) -> Result<Self, Box<dyn Error>>
    where
        T: ToSocketAddrs,
    {
        let (reader, writer) = TcpStream::connect(address).await?.into_split();

        Ok(Self {
            reader: NodeToClientMessageStreamReader::new(reader),
            writer: ClientToNodeMessageStreamWriter::new(writer),
            next_dispatch_id: 0,
        })
    }

    pub async fn load_module(&mut self, path: &Path) -> Result<u64, Box<dyn Error>> {
        let file = File::open(path).await?;

        self.writer
            .write(ClientToNodeMessage::InitializeRequest(
                ClientToNodeInitializeRequestMessage::new(),
            ))
            .await?;

        let module_id = match self.reader.read().await? {
            None => return Err("Expected response".into()),
            Some(NodeToClientMessage::InitializeResponse(message)) => message.module_id(),
            Some(_) => return Err("Expected response".into()),
        };

        let mut stream = ReaderStream::with_capacity(file, 4096 * 4);

        while let Some(bytes) = stream.next().await {
            let bytes = bytes?;

            if bytes.is_empty() {
                break;
            }

            self.writer
                .write(ClientToNodeMessage::Append(ClientToNodeAppendMessage::new(
                    module_id, bytes,
                )))
                .await?;
        }

        self.writer
            .write(ClientToNodeMessage::LoadRequest(
                ClientToNodeLoadRequestMessage::new(module_id),
            ))
            .await?;

        match self.reader.read().await? {
            None => return Err("Expected response".into()),
            Some(NodeToClientMessage::LoadResponse(message)) => {
                let (_, result) = message.into();

                result?
            }
            Some(_) => return Err("Expected response".into()),
        };

        Ok(module_id)
    }

    pub async fn dispatch<T>(
        &mut self,
        module_id: ModuleId,
        reader: T,
    ) -> Result<Bytes, Box<dyn Error>>
    where
        T: AsyncRead + Unpin,
    {
        let mut stream = ReaderStream::new(reader);
        let mut payload = BytesMut::new();

        while let Some(bytes) = stream.next().await {
            let bytes = bytes?;

            if bytes.is_empty() {
                break;
            }

            payload.extend(&bytes);
        }

        self.writer
            .write(ClientToNodeMessage::DispatchRequest(
                ClientToNodeDispatchRequestMessage::new(
                    module_id,
                    self.next_dispatch_id,
                    payload.into(),
                    None,
                ),
            ))
            .await?;

        self.next_dispatch_id += 1;

        let result = match self.reader.read().await? {
            None => return Err("Expected response".into()),
            Some(NodeToClientMessage::DispatchResponse {
                dispatch_id: _,
                result,
            }) => result,
            Some(_) => return Err("Expected response".into()),
        };

        let payload = result?;

        Ok(payload)
    }
}
