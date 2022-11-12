use bytes::Bytes;
use futures::StreamExt;
use std::io;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::{select, spawn};
use tokio_util::io::ReaderStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum TcpListenerMessage {
    UpdateSender(Sender<Result<(TcpStream, SocketAddr), io::Error>>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct TcpListenerActor {
    sender: Sender<Result<(TcpStream, SocketAddr), io::Error>>,
    receiver: Receiver<TcpListenerMessage>,
    listener: TcpListener,
}

impl TcpListenerActor {
    pub fn new(
        tcp_stream_sender: Sender<Result<(TcpStream, SocketAddr), io::Error>>,
        listener: TcpListener,
    ) -> Sender<TcpListenerMessage> {
        let (sender, receiver) = channel(32);

        spawn(async move {
            (Self {
                sender: tcp_stream_sender,
                receiver,
                listener,
            })
            .main()
            .await
        });

        sender
    }

    async fn main(&mut self) {
        loop {
            select!(
                result = self.listener.accept() => {
                    if self.sender.send(result).await.is_err() {
                        break
                    }
                }
                message = self.receiver.recv() => {
                    let message = match message {
                        None => break,
                        Some(message) => message,
                    };

                    match message {
                        TcpListenerMessage::UpdateSender(sender) => {
                            self.sender = sender;
                        }
                    };
                }
            )
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum TcpStreamMessage {
    UpdateSender(Sender<Result<Bytes, io::Error>>),
    SendBytes(Bytes),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TcpStreamActor {
    sender: Sender<Result<Bytes, io::Error>>,
    receiver: Receiver<TcpStreamMessage>,
    reader: ReaderStream<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

impl TcpStreamActor {
    pub fn new(
        sender: Sender<Result<Bytes, io::Error>>,
        stream: TcpStream,
    ) -> Sender<TcpStreamMessage> {
        let (tx, receiver) = channel(32);

        spawn(async move {
            let (reader, writer) = stream.into_split();

            (Self {
                sender,
                receiver,
                reader: ReaderStream::new(reader),
                writer,
            })
            .main()
            .await
        });

        tx
    }

    async fn main(&mut self) {
        loop {
            select!(
                item = self.reader.next() => {
                    let result = match item {
                        None => break,
                        Some(result) => result,
                    };

                    if self.sender.send(result).await.is_err() {
                        break;
                    }
                }
                message = self.receiver.recv() => {
                    let message = match message {
                        None => break,
                        Some(message) => message,
                    };

                    match message {
                        TcpStreamMessage::UpdateSender(sender) => {
                            self.sender = sender;
                        }
                        TcpStreamMessage::SendBytes(bytes) => {
                            if self.writer.write(&bytes).await.is_err() {
                                break;
                            }
                        }
                    };
                }
            )
        }
    }
}
