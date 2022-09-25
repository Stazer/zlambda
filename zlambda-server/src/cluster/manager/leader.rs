use crate::cluster::manager::ManagerFollowerState;
use crate::cluster::packet::{from_bytes, to_vec, Packet, ReadPacketError};
use std::collections::HashMap;
use crate::common::{
    ActorStopMessage,
    TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor, TcpStreamActorReceiveMessage,
    TcpStreamActorSendMessage,
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, WrapFuture,
};
use bytes::Bytes;
use std::io;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

////////////////////////////////////////////////////////////////////////////////////////////////////

struct ManagerLeaderFollowerActor {
    id: usize,
    buffer: Vec<u8>,
    leader: Addr<ManagerLeaderActor>,
    stream: Addr<TcpStreamActor>,
    state: ManagerFollowerState,
}

impl Actor for ManagerLeaderFollowerActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {
        self.stream.do_send(ActorStopMessage);

        self.leader.do_send(ManagerLeaderActorRemoveFollowerMessage {
            id: self.id,
        });
    }
}

impl Handler<ActorStopMessage> for ManagerLeaderFollowerActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(&mut self, _: ActorStopMessage, context: &mut <Self as Actor>::Context) -> Self::Result {
        context.stop();
    }
}

impl Handler<TcpStreamActorReceiveMessage> for ManagerLeaderFollowerActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result,) = message.into();

        let bytes = match result {
            Err(e) => {
                eprintln!("wat ze {}", e);
                context.stop();
                return;
            }
            Ok(b) => b,
        };

        self.buffer.extend(bytes);

        loop {
            let (read, packet) = match from_bytes(&self.buffer) {
                Err(ReadPacketError::UnexpectedEnd) => {
                    break;
                }
                Err(e) => {
                    eprintln!("wuuuut {}", e);
                    context.stop();
                    return;
                }
                Ok((r, p)) => (r, p),
            };

            self.buffer.drain(0..read);

            self.handle_packet(packet, context);
        }
    }
}

impl ManagerLeaderFollowerActor {
    fn new(id: usize, leader: Addr<ManagerLeaderActor>, stream: TcpStream) -> Addr<Self> {
        Self::create(move |context| Self {
            leader,
            id,
            stream: TcpStreamActor::new(context.address().recipient(), stream),
            buffer: Vec::default(),
            state: ManagerFollowerState::Handshaking,
        })
    }

    fn handle_packet(&mut self, packet: Packet, context: &mut <Self as Actor>::Context) {
        match self.state {
            ManagerFollowerState::Handshaking => self.handle_handshaking_packet(packet, context),
            ManagerFollowerState::Operational => self.handle_operational_packet(packet, context),
        }
    }

    fn handle_handshaking_packet(
        &mut self,
        packet: Packet,
        context: &mut <Self as Actor>::Context,
    ) {
        match packet {
            Packet::ManagerFollowerHandshakeChallenge => {
                self.state = ManagerFollowerState::Operational;

                context.run_interval(Duration::SECOND, |actor, context| {
                    let stream = actor.stream.clone();

                    let bytes = match to_vec(&Packet::Ping) {
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
                        .into_actor(actor)
                        .map(|result, _, context| match result {
                            Err(_) => {
                                context.stop();
                            }
                            Ok(Err(e)) => {
                                eprintln!("lul2 {}", e);
                                context.stop();
                            }
                            Ok(Ok(())) => (),
                        }),
                    );
                });

                let bytes = match to_vec(&Packet::ManagerFollowerHandshakeSuccess) {
                    Err(e) => {
                        eprintln!("{}", e);
                        context.stop();
                        return;
                    }
                    Ok(b) => b,
                };

                let stream = self.stream.clone();

                context.wait(
                    async move {
                        stream
                            .send(TcpStreamActorSendMessage::new(Bytes::from(bytes)))
                            .await
                    }
                    .into_actor(self)
                    .map(|result, _, context| match result {
                        Err(e) => {
                            eprintln!("hui {}", e);
                            context.stop();
                        }
                        Ok(Err(e)) => {
                            eprintln!("hui2 {}", e);
                            context.stop();
                        }
                        Ok(Ok(())) => (),
                    }),
                );
            }
            _ => {
                eprintln!("Unhandled handshake packet {:?}", packet);
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
            Packet::Pong => {
                println!("Received pong from {}", self.id);
            }
            _ => {
                eprintln!("Unhandled packet {:?}", packet);
                context.stop();
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct ManagerLeaderActorRemoveFollowerMessage {
    id: usize,
}

impl Message for ManagerLeaderActorRemoveFollowerMessage {
    type Result = ();
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ManagerLeaderActor {
    listener: Addr<TcpListenerActor>,
    followers_counter: usize,
    followers: HashMap<usize, Addr<ManagerLeaderFollowerActor>>,
}

impl Actor for ManagerLeaderActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut <Self as Actor>::Context) {
        self.listener.do_send(ActorStopMessage);

        for value in self.followers.values() {
            value.do_send(ActorStopMessage);
        }
    }
}

impl Handler<TcpListenerActorAcceptMessage> for ManagerLeaderActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result,) = message.into();

        let stream = match result {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        };

        self.followers
            .insert(self.followers_counter, ManagerLeaderFollowerActor::new(self.followers_counter, context.address(), stream));

        self.followers_counter = self.followers_counter + 1;

    }
}

impl Handler<ManagerLeaderActorRemoveFollowerMessage> for ManagerLeaderActor {
    type Result = <ManagerLeaderActorRemoveFollowerMessage as Message>::Result;

    fn handle(&mut self, message: ManagerLeaderActorRemoveFollowerMessage, _: &mut <Self as Actor>::Context) -> Self::Result {
        self.followers.remove(&message.id);
    }
}

impl ManagerLeaderActor {
    pub async fn new<T>(listener_address: T) -> Result<Addr<Self>, io::Error>
    where
        T: ToSocketAddrs,
    {
        let listener = TcpListener::bind(listener_address).await?;

        Ok(Self::create(|context| Self {
            listener: TcpListenerActor::new(context.address().recipient(), listener),
            followers: HashMap::default(),
            followers_counter: 0,
        }))
    }
}
