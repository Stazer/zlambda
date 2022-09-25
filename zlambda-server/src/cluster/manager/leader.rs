use crate::cluster::manager::{ManagerFollowerState, ManagerId};
use crate::cluster::packet::{Packet, ReadPacketError};
use crate::common::{
    ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor,
    TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, WrapFuture,
};
use bytes::Bytes;
use std::collections::HashMap;
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

////////////////////////////////////////////////////////////////////////////////////////////////////

struct ManagerLeaderReaderActor {
    id: ManagerId,
    buffer: Vec<u8>,
    leader: Addr<ManagerLeaderActor>,
    stream: Addr<TcpStreamActor>,
}

impl Actor for ManagerLeaderReaderActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {
        self.stream.do_send(ActorStopMessage);

        self.leader
            .do_send(ManagerLeaderActorRemoveFollowerMessage { id: self.id });
    }
}

impl Handler<ActorStopMessage> for ManagerLeaderReaderActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl Handler<TcpStreamActorReceiveMessage> for ManagerLeaderReaderActor {
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
            let (read, packet) = match Packet::from_vec(&self.buffer) {
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

            self.leader.do_send(ManagerLeaderActorReceivePacketMessage {
                id: self.id,
                packet,
            });
        }
    }
}

impl ManagerLeaderReaderActor {
    fn new(id: ManagerId, leader: Addr<ManagerLeaderActor>, stream: TcpStream) -> (Addr<Self>, Addr<TcpStreamActor>) {
        let context = Context::new();
        let stream = TcpStreamActor::new(context.address().recipient(), stream);

        (context.run(Self {
            leader,
            id,
            stream: stream.clone(),
            buffer: Vec::default(),
        }), stream.clone())
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct ManagerLeaderActorRemoveFollowerMessage {
    id: ManagerId,
}

impl Message for ManagerLeaderActorRemoveFollowerMessage {
    type Result = ();
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct ManagerLeaderActorReceivePacketMessage {
    id: ManagerId,
    packet: Packet,
}

impl Message for ManagerLeaderActorReceivePacketMessage {
    type Result = ();
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct ManagerLeaderActorFollowerEntry {
    id: ManagerId,
    reader: Addr<ManagerLeaderReaderActor>,
    stream: Addr<TcpStreamActor>,
    state: ManagerFollowerState,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ManagerLeaderActor {
    listener: Addr<TcpListenerActor>,
    followers_counter: ManagerId,
    followers: HashMap<ManagerId, ManagerLeaderActorFollowerEntry>,
}

impl Actor for ManagerLeaderActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut <Self as Actor>::Context) {
        self.listener.do_send(ActorStopMessage);

        for value in self.followers.values() {
            value.reader.do_send(ActorStopMessage);
            value.stream.do_send(ActorStopMessage);
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

        self.followers_counter = self.followers_counter + 1;

        let (reader, stream) =
            ManagerLeaderReaderActor::new(self.followers_counter, context.address(), stream);

        self.followers.insert(
            self.followers_counter,
            ManagerLeaderActorFollowerEntry {
                id: self.followers_counter,
                reader,
                stream,
                state: ManagerFollowerState::Handshaking,
            },
        );
    }
}

impl Handler<ManagerLeaderActorReceivePacketMessage> for ManagerLeaderActor {
    type Result = <ManagerLeaderActorReceivePacketMessage as Message>::Result;

    fn handle(
        &mut self,
        message: ManagerLeaderActorReceivePacketMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        self.handle_packet(message.id, message.packet, context);
    }
}

impl Handler<ManagerLeaderActorRemoveFollowerMessage> for ManagerLeaderActor {
    type Result = <ManagerLeaderActorRemoveFollowerMessage as Message>::Result;

    fn handle(
        &mut self,
        message: ManagerLeaderActorRemoveFollowerMessage,
        _: &mut <Self as Actor>::Context,
    ) -> Self::Result {
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

    fn handle_packet(
        &mut self,
        id: ManagerId,
        packet: Packet,
        context: &mut <Self as Actor>::Context,
    ) {
        let state = match self.followers.get(&id) {
            Some(f) => &f.state,
            None => {
                eprintln!("{} not found", id);
                return;
            }
        };

        match state {
            ManagerFollowerState::Handshaking => self.handle_handshaking_packet(id, packet, context),
            ManagerFollowerState::Operational => self.handle_operational_packet(id, packet, context),
        };
    }

    fn handle_handshaking_packet(
        &mut self,
        id: ManagerId,
        packet: Packet,
        context: &mut <Self as Actor>::Context,
    ) {
        match packet {
            Packet::ManagerFollowerHandshakeChallenge {
                listener_local_address,
            } => {
                let follower = match self.followers.get_mut(&id) {
                    Some(f) => f,
                    None => {
                        eprintln!("{} not found", &id);
                        return;
                    }
                };

                follower.state = ManagerFollowerState::Operational;

                let stream = follower.stream.clone();

                context.run_interval(Duration::SECOND, move |actor, context| {
                    let stream = stream.clone();

                    let bytes = match Packet::Ping.to_vec() {
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

                let bytes = match Packet::ManagerFollowerHandshakeSuccess.to_vec() {
                    Err(e) => {
                        eprintln!("{}", e);
                        context.stop();
                        return;
                    }
                    Ok(b) => b,
                };

                let stream = follower.stream.clone();

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
        id: ManagerId,
        packet: Packet,
        context: &mut <Self as Actor>::Context,
    ) {
        match packet {
            Packet::Pong => {
                println!("Received pong from {}", id);
            }
            _ => {
                eprintln!("Unhandled packet {:?}", packet);
                context.stop();
            }
        }
    }
}
