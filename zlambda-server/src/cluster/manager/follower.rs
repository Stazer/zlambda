use crate::cluster::packet::{from_bytes, to_vec, Packet, ReadPacketError};
use crate::common::{TcpStreamActor, TcpStreamActorReceiveMessage, TcpStreamActorSendMessage};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message,
    WrapFuture,
};
use bytes::{Bytes};
use std::io;
use tokio::net::{TcpStream, ToSocketAddrs};

////////////////////////////////////////////////////////////////////////////////////////////////////

enum ManagerFollowerActorState {
    Handshake,
    Operational,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ManagerFollowerActor {
    follower: Addr<TcpStreamActor>,
    buffer: Vec<u8>,
    state: ManagerFollowerActorState,
}

impl Actor for ManagerFollowerActor {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        let follower = self.follower.clone();

        let packet = match to_vec(&Packet::ManagerFollowerHandshakeChallenge) {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{}", e);
                context.stop();

                return;
            }
        };

        context.wait(
            async move {
                follower
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
            let (read, packet) = match from_bytes(&self.buffer) {
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
    pub async fn new<S, T>(
        _listener_address: S,
        follower_address: T,
    ) -> Result<Addr<Self>, io::Error>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let stream = TcpStream::connect(follower_address).await?;

        Ok(Self::create(move |context| Self {
            follower: TcpStreamActor::new(context.address().recipient(), stream),
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
                let stream = self.follower.clone();

                let bytes = match to_vec(&Packet::Pong) {
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
