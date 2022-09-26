use crate::cluster::packet::Packet;
use crate::common::{
    ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor,
    TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message, Recipient};
use bytes::BytesMut;
use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ManagerClientId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ManagerClientActorReadPacketMessage {
    id: ManagerClientId,
    packet: Packet,
}

impl From<ManagerClientActorReadPacketMessage> for (ManagerClientId, Packet) {
    fn from(message: ManagerClientActorReadPacketMessage) -> Self {
        (message.id, message.packet)
    }
}

impl Message for ManagerClientActorReadPacketMessage {
    type Result = ();
}

impl ManagerClientActorReadPacketMessage {
    pub fn new(id: ManagerClientId, packet: Packet) -> Self {
        Self { id, packet }
    }

    pub fn id(&self) -> ManagerClientId {
        self.id
    }

    pub fn packet(&self) -> &Packet {
        &self.packet
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ManagerClientActor {
    id: ManagerClientId,
    recipient: Addr<ManagerActor>,
    stream: Addr<TcpStreamActor>,
    buffer: BytesMut,
}

impl Actor for ManagerClientActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {
        self.stream.do_send(ActorStopMessage);
        self.recipient
            .do_send(ManagerActorRemoveClientMessage::new(self.id));
    }
}

impl Handler<ActorStopMessage> for ManagerClientActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl Handler<TcpStreamActorReceiveMessage> for ManagerClientActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result, ) = message.into();

        let bytes = match result {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{:?}", e);
                context.stop();
                return;
            }
        };

        self.buffer.extend(bytes);

        let (read, packet) = match Packet::from_bytes(&self.buffer) {
            Ok((r, p)) => (r, p),
            Err(e) => {
                eprintln!("{:?}", e);
                context.stop();
                return;
            }
        };

        self.recipient.do_send(
            ManagerClientActorReadPacketMessage::new(self.id, packet)
        );
    }
}

impl ManagerClientActor {
    pub fn new(
        id: ManagerClientId,
        recipient: Addr<ManagerActor>,
        stream: TcpStream,
    ) -> Addr<Self> {
        Self::create(move |context| Self {
            id,
            recipient,
            stream: TcpStreamActor::new(
                context.address().recipient(),
                Some(context.address().recipient()),
                stream,
            ),
            buffer: BytesMut::default(),
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type ManagerId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum ManagerType {
    Leader {},
    Follower { stream: Addr<TcpStreamActor> },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ManagerClient {
    id: ManagerClientId,
    address: Addr<ManagerClientActor>,
}

impl ManagerClient {
    pub fn new(id: ManagerClientId, address: Addr<ManagerClientActor>) -> Self {
        Self { id, address }
    }

    pub fn id(&self) -> ManagerClientId {
        self.id
    }

    pub fn address(&self) -> &Addr<ManagerClientActor> {
        &self.address
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ManagerActorRemoveClientMessage {
    id: ManagerClientId,
}

impl From<ManagerActorRemoveClientMessage> for (ManagerClientId,) {
    fn from(message: ManagerActorRemoveClientMessage) -> Self {
        (message.id,)
    }
}

impl Message for ManagerActorRemoveClientMessage {
    type Result = ();
}

impl ManagerActorRemoveClientMessage {
    pub fn new(id: ManagerClientId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> ManagerClientId {
        self.id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ManagerActor {
    id: ManagerId,
    listener: Addr<TcpListenerActor>,
    r#type: ManagerType,
    clients: HashMap<ManagerClientId, ManagerClient>,
}

impl Actor for ManagerActor {
    type Context = Context<Self>;

    fn started(&mut self, _context: &mut Self::Context) {
        match self.r#type {
            ManagerType::Follower { stream, .. } => {
            },
            _ => {},
        };
    }

    fn stopped(&mut self, _context: &mut Self::Context) {
        self.listener.do_send(ActorStopMessage);

        match &self.r#type {
            ManagerType::Leader {} => {}
            ManagerType::Follower { stream, .. } => {
                stream.do_send(ActorStopMessage);
            }
        }
    }
}

impl Handler<ActorStopMessage> for ManagerActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _message: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl Handler<ManagerClientActorReadPacketMessage> for ManagerActor {
    type Result = <ManagerClientActorReadPacketMessage as Message>::Result;

    fn handle(
        &mut self,
        message: ManagerClientActorReadPacketMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (_id, _packet) = message.into();
    }
}

impl Handler<ManagerActorRemoveClientMessage> for ManagerActor {
    type Result = <ManagerActorRemoveClientMessage as Message>::Result;

    fn handle(
        &mut self,
        message: ManagerActorRemoveClientMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        println!("Byebye {}", message.id());
        self.clients.remove(&message.id());
    }
}

impl Handler<TcpListenerActorAcceptMessage> for ManagerActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let mut keys = self.clients.keys().map(|x| *x as u64).collect::<Vec<_>>();
        keys.sort_unstable();
        let mut id = 0;
        for key in keys {
            if id != key {
                break;
            }

            id = id + 1;
        }

        let (stream,) = message.into();
        self.clients.insert(
            id,
            ManagerClient::new(
                id,
                ManagerClientActor::new(id, context.address(), stream.unwrap()),
            ),
        );

        println!("{} connected", id);
    }
}

impl Handler<TcpStreamActorReceiveMessage> for ManagerActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let _ = match self.r#type {
            ManagerType::Follower { ref stream } => stream,
            _ => {
                panic!("TcpStreamActorReceiveMessage should be never received by ManagerActor with type {:?}", self.r#type);
            }
        };
    }
}

impl ManagerActor {
    pub async fn new<S, T>(
        listener_address: S,
        stream_address: Option<T>,
    ) -> Result<Addr<Self>, io::Error>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        let listener = TcpListener::bind(listener_address).await?;

        match stream_address {
            Some(stream_address) => {
                let stream = TcpStream::connect(stream_address).await?;

                Ok(Self::create(move |context| Self {
                    id: 0,
                    listener: TcpListenerActor::new(
                        context.address().recipient(),
                        Some(context.address().recipient()),
                        listener,
                    ),
                    r#type: ManagerType::Follower {
                        stream: TcpStreamActor::new(
                            context.address().recipient(),
                            Some(context.address().recipient()),
                            stream,
                        ),
                    },
                    clients: HashMap::default(),
                }))
            }
            None => Ok(Self::create(move |context| Self {
                id: 0,
                listener: TcpListenerActor::new(
                    context.address().recipient(),
                    Some(context.address().recipient()),
                    listener,
                ),
                r#type: ManagerType::Leader {},
                clients: HashMap::default(),
            })),
        }
    }
}
