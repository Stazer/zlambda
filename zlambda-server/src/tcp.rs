use bytes::Bytes;
use futures::StreamExt;
use std::io;
use std::net::SocketAddr;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::{select, spawn};
use tokio_util::io::ReaderStream;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type TcpListenerIncomingMessage = ();
pub type TcpListenerIncomingSender = mpsc::Sender<TcpListenerIncomingMessage>;
pub type TcpListenerIncomingReceiver = mpsc::Receiver<TcpListenerIncomingMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn tcp_listener_incoming_message_channel(
) -> (TcpListenerIncomingSender, TcpListenerIncomingReceiver) {
    mpsc::channel(16)
}
////////////////////////////////////////////////////////////////////////////////////////////////////

pub type TcpListenerOutgoingMessage = Result<(TcpStream, SocketAddr), io::Error>;
pub type TcpListenerOutgoingSender = mpsc::Sender<TcpListenerOutgoingMessage>;
pub type TcpListenerOutgoingReceiver = mpsc::Receiver<TcpListenerOutgoingMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn tcp_listener_outgoing_message_channel(
) -> (TcpListenerOutgoingSender, TcpListenerOutgoingReceiver) {
    mpsc::channel(16)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct TcpListenerActor {
    incoming_receiver: TcpListenerIncomingReceiver,
    outgoing_sender: TcpListenerOutgoingSender,
    listener: TcpListener,
}

impl TcpListenerActor {
    pub fn new(listener: TcpListener) -> (TcpListenerIncomingSender, TcpListenerOutgoingReceiver) {
        let (incoming_sender, incoming_receiver) = tcp_listener_incoming_message_channel();
        let (outgoing_sender, outgoing_receiver) = tcp_listener_outgoing_message_channel();

        spawn(async move {
            (Self {
                listener,
                incoming_receiver,
                outgoing_sender,
            })
            .main()
            .await
        });

        (incoming_sender, outgoing_receiver)
    }

    async fn main(&mut self) {
        loop {
            select!(
                result = self.listener.accept() => {
                    if self.outgoing_sender.send(result).await.is_err() {
                        break
                    }
                }
                message = self.incoming_receiver.recv() => {
                    let message = match message {
                        None => break,
                        Some(message) => message,
                    };

                    match message {
                        _ => todo!(),
                    };
                }
            )
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum TcpStreamMessage {
    SendBytes(Bytes),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type TcpStreamMessageSender = mpsc::Sender<TcpStreamMessage>;
pub type TcpStreamMessageReceiver = mpsc::Receiver<TcpStreamMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type TcpStreamResult = Result<Bytes, io::Error>;
pub type TcpStreamResultSender = mpsc::Sender<TcpStreamResult>;
pub type TcpStreamResultReceiver = mpsc::Receiver<TcpStreamResult>;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum TcpStreamIncomingMessage {
    SendBytes(Bytes),
}

pub type TcpStreamIncomingSender = mpsc::Sender<TcpStreamIncomingMessage>;
pub type TcpStreamIncomingReceiver = mpsc::Receiver<TcpStreamIncomingMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn tcp_stream_incoming_message_channel() -> (TcpStreamIncomingSender, TcpStreamIncomingReceiver)
{
    mpsc::channel(16)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type TcpStreamOutgoingMessage = Result<Bytes, io::Error>;
pub type TcpStreamOutgoingSender = mpsc::Sender<TcpStreamOutgoingMessage>;
pub type TcpStreamOutgoingReceiver = mpsc::Receiver<TcpStreamOutgoingMessage>;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn tcp_stream_outgoing_message_channel() -> (TcpStreamOutgoingSender, TcpStreamOutgoingReceiver)
{
    mpsc::channel(16)
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct TcpStreamActor {
    outgoing_sender: TcpStreamOutgoingSender,
    incoming_receiver: TcpStreamIncomingReceiver,
    reader: ReaderStream<OwnedReadHalf>,
    writer: OwnedWriteHalf,
}

impl TcpStreamActor {
    pub fn new(stream: TcpStream) -> (TcpStreamIncomingSender, TcpStreamOutgoingReceiver) {
        let (incoming_sender, incoming_receiver) = tcp_stream_incoming_message_channel();
        let (outgoing_sender, outgoing_receiver) = tcp_stream_outgoing_message_channel();
        let (reader, writer) = stream.into_split();

        spawn(async move {
            (Self {
                outgoing_sender,
                incoming_receiver,
                reader: ReaderStream::new(reader),
                writer,
            })
            .main()
            .await
        });

        (incoming_sender, outgoing_receiver)
    }

    async fn main(&mut self) {
        loop {
            select!(
                item = self.reader.next() => {
                    let result = match item {
                        None => break,
                        Some(result) => result,
                    };

                    if self.outgoing_sender.send(result).await.is_err() {
                        break;
                    }
                }
                message = self.incoming_receiver.recv() => {
                    let message = match message {
                        None => break,
                        Some(message) => message,
                    };

                    match message {
                        TcpStreamIncomingMessage::SendBytes(bytes) => {
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
