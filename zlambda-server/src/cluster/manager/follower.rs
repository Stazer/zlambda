use crate::cluster::packet::{Packet, ReadPacketError};
use crate::common::{
    ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor,
    TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, WrapFuture,
};
use bytes::Bytes;
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

////////////////////////////////////////////////////////////////////////////////////////////////////

enum ManagerFollowerActorState {
    Handshake,
    Operational,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ManagerFollowerActor {
    listener: Addr<TcpListenerActor>,
    listener_local_address: SocketAddr,

    stream: Addr<TcpStreamActor>,
    buffer: Vec<u8>,
    state: ManagerFollowerActorState,
}

impl Actor for ManagerFollowerActor {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        let stream = self.stream.clone();

        let packet = match (Packet::ManagerFollowerHandshakeChallenge {
            listener_local_address: self.listener_local_address.clone(),
        })
        .to_vec()
        {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}", e);
                context.stop();

                return;
            }
        };

        context.wait(
            async move {
                stream
                    .send(TcpStreamActorSendMessage::new(Bytes::from(packet)))
                    .await
            }
            .into_actor(self)
            .map(|result, _actor, context| match result {
                Err(e) => {
                    eprintln!("{}", e);
                    context.stop();
                }
                Ok(Err(e)) => {
                    eprintln!("{}", e);
                    context.stop();
                }
                Ok(Ok(())) => (),
            }),
        );
    }

    fn stopped(&mut self, _context: &mut Self::Context) {
        self.listener.do_send(ActorStopMessage);
        self.stream.do_send(ActorStopMessage);
    }
}

impl Handler<TcpListenerActorAcceptMessage> for ManagerFollowerActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    fn handle(
        &mut self,
        _message: TcpListenerActorAcceptMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
    }
}

impl Handler<TcpStreamActorReceiveMessage> for ManagerFollowerActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let bytes = match message.result() {
            Err(e) => {
                eprintln!("{}", e);
                context.stop();

                return;
            }
            Ok(b) => b,
        };

        self.buffer.extend(bytes);

        loop {
            let (read, packet) = match Packet::from_vec(&self.buffer) {
                Err(ReadPacketError::UnexpectedEnd) => break,
                Err(e) => {
                    eprintln!("{}", e);
                    context.stop();

                    return;
                }
                Ok(p) => p,
            };

            self.buffer.drain(0..read);

            self.handle_packet(packet, context);
        }
    }
}

impl ManagerFollowerActor {
    pub async fn new<S, T>(listener_address: S, stream_address: T) -> Result<Addr<Self>, io::Error>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let listener = TcpListener::bind(listener_address).await?;
        let listener_local_address = listener.local_addr()?;
        let stream = TcpStream::connect(stream_address).await?;

        Ok(Self::create(move |context| Self {
            listener: TcpListenerActor::new(context.address().recipient(), listener),
            listener_local_address,
            stream: TcpStreamActor::new(context.address().recipient(), stream),
            state: ManagerFollowerActorState::Handshake,
            buffer: Vec::default(),
        }))
    }

    fn handle_packet(&mut self, packet: Packet, context: &mut <Self as Actor>::Context) {
        match self.state {
            ManagerFollowerActorState::Handshake => self.handle_handshaking_packet(packet, context),
            ManagerFollowerActorState::Operational => {
                self.handle_operational_packet(packet, context)
            }
        }
    }

    fn handle_handshaking_packet(
        &mut self,
        packet: Packet,
        context: &mut <Self as Actor>::Context,
    ) {
        match packet {
            Packet::ManagerFollowerHandshakeSuccess => {
                self.state = ManagerFollowerActorState::Operational;
            }
            _ => {
                eprintln!("Unknown packet in handshake state {:?}", packet);
                context.stop();
            }
        }
    }

    fn handle_operational_packet(
        &mut self,
        packet: Packet,
        context: &mut <Self as Actor>::Context,
    ) {
        match packet {
            Packet::Ping => {
                let stream = self.stream.clone();

                let bytes = match Packet::Pong.to_vec() {
                    Err(e) => {
                        eprintln!("{}", e);
                        context.stop();
                        return;
                    }
                    Ok(b) => b,
                };

                context.wait(
                    async move {
                        stream
                            .send(TcpStreamActorSendMessage::new(Bytes::from(bytes)))
                            .await
                    }
                    .into_actor(self)
                    .map(|result, _, context| match result {
                        Err(e) => {
                            eprintln!("{}", e);
                            context.stop();
                        }
                        Ok(Err(e)) => {
                            eprintln!("{}", e);
                            context.stop();
                        }
                        Ok(Ok(())) => (),
                    }),
                );
            }
            _ => {
                eprintln!("Unknown packet in handshake state {:?}", packet);
                context.stop();
            }
        }
    }
}
