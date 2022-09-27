use crate::algorithm::next_key;
use crate::cluster::packet::{Packet, ReadPacketError};
use crate::common::{
    ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage, TcpStreamActor,
    TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::collections::HashMap;
use std::io;
use std::rc::Rc;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};
use tracing_subscriber::fmt::init;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NodeClientId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct PacketReaderActorReadPacketMessage {
    id: NodeClientId,
    packet: Packet,
}

impl From<PacketReaderActorReadPacketMessage> for (NodeClientId, Packet) {
    fn from(message: PacketReaderActorReadPacketMessage) -> Self {
        (message.id, message.packet)
    }
}

impl Message for PacketReaderActorReadPacketMessage {
    type Result = ();
}

impl PacketReaderActorReadPacketMessage {
    pub fn new(id: NodeClientId, packet: Packet) -> Self {
        Self { id, packet }
    }

    pub fn id(&self) -> NodeClientId {
        self.id
    }

    pub fn packet(&self) -> &Packet {
        &self.packet
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct PacketReaderActor {
    id: NodeClientId,
    recipient: Addr<NodeActor>,
    stream: Addr<TcpStreamActor>,
    buffer: Vec<u8>,
}

impl Actor for PacketReaderActor {
    type Context = Context<Self>;

    #[tracing::instrument]
    fn stopped(&mut self, _: &mut Self::Context) {
        self.stream.do_send(ActorStopMessage);
        self.recipient
            .do_send(NodeActorRemoveClientMessage::new(self.id));
    }
}

impl Handler<ActorStopMessage> for PacketReaderActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl Handler<TcpStreamActorReceiveMessage> for PacketReaderActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (result,) = message.into();

        let bytes = match result {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{:?}", e);
                context.stop();
                return;
            }
        };

        self.buffer.extend(bytes);

        loop {
            let (read, packet) = match Packet::from_vec(&self.buffer) {
                Ok((r, p)) => (r, p),
                Err(ReadPacketError::UnexpectedEnd) => break,
                Err(e) => {
                    eprintln!("{:?}", e);
                    context.stop();
                    return;
                }
            };

            self.buffer.drain(0..read);

            self.recipient
                .do_send(PacketReaderActorReadPacketMessage::new(self.id, packet));
        }
    }
}

impl PacketReaderActor {
    pub fn new(
        id: NodeClientId,
        recipient: Addr<NodeActor>,
        stream: TcpStream,
    ) -> (Addr<Self>, Addr<TcpStreamActor>) {
        let context = Context::new();
        let stream = TcpStreamActor::new(
            context.address().recipient(),
            Some(context.address().recipient()),
            stream,
        );

        (
            context.run(Self {
                id,
                recipient,
                stream: stream.clone(),
                buffer: Vec::default(),
            }),
            stream,
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NodeId = u64;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeType {
    Leader {},
    Follower {
        stream: Addr<TcpStreamActor>,
        buffer: Vec<u8>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub enum NodeClientType {
    User {},
    Follower(Rc<NodeFollower>),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeClient {
    id: NodeClientId,
    reader: Addr<PacketReaderActor>,
    stream: Addr<TcpStreamActor>,
    r#type: Option<NodeClientType>,
}

impl NodeClient {
    pub fn new(
        id: NodeClientId,
        reader: Addr<PacketReaderActor>,
        stream: Addr<TcpStreamActor>,
    ) -> Self {
        Self {
            id,
            reader,
            stream,
            r#type: None,
        }
    }

    pub fn id(&self) -> NodeClientId {
        self.id
    }

    pub fn r#type(&self) -> &Option<NodeClientType> {
        &self.r#type
    }

    pub fn set_type(&mut self, r#type: Option<NodeClientType>) {
        self.r#type = r#type;
    }

    pub fn reader(&self) -> &Addr<PacketReaderActor> {
        &self.reader
    }

    pub fn stream(&self) -> &Addr<TcpStreamActor> {
        &self.stream
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeFollower {
    id: NodeId,
}

impl NodeFollower {
    pub fn new(id: NodeId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> NodeId {
        self.id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActorRemoveClientMessage {
    id: NodeClientId,
}

impl From<NodeActorRemoveClientMessage> for (NodeClientId,) {
    fn from(message: NodeActorRemoveClientMessage) -> Self {
        (message.id,)
    }
}

impl Message for NodeActorRemoveClientMessage {
    type Result = ();
}

impl NodeActorRemoveClientMessage {
    pub fn new(id: NodeClientId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> NodeClientId {
        self.id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActor {
    id: NodeId,
    listener: Addr<TcpListenerActor>,
    r#type: NodeType,
    clients: HashMap<NodeClientId, NodeClient>,
    followers: HashMap<NodeId, Rc<NodeFollower>>,
}

impl Actor for NodeActor {
    type Context = Context<Self>;

    #[tracing::instrument]
    fn started(&mut self, context: &mut Self::Context) {
        match self.r#type {
            NodeType::Follower { ref stream, .. } => {
                let packet = match Packet::FollowerHandshakeChallenge.to_vec() {
                    Err(e) => {
                        eprintln!("{}", e);
                        context.stop();
                        return;
                    }
                    Ok(p) => p,
                };

                stream.do_send(TcpStreamActorSendMessage::new(packet.into()));
            }
            _ => {}
        };
    }

    #[tracing::instrument]
    fn stopped(&mut self, _context: &mut Self::Context) {
        self.listener.do_send(ActorStopMessage);

        for client in self.clients.values() {
            client.reader().do_send(ActorStopMessage);
            client.stream().do_send(ActorStopMessage);
        }

        match &self.r#type {
            NodeType::Leader {} => {}
            NodeType::Follower { stream, .. } => {
                stream.do_send(ActorStopMessage);
            }
        }
    }
}

impl Handler<ActorStopMessage> for NodeActor {
    type Result = <ActorStopMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        _message: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}

impl Handler<PacketReaderActorReadPacketMessage> for NodeActor {
    type Result = <PacketReaderActorReadPacketMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        trace!("Received packet");
        let (id, packet) = message.into();

        let client = match self.clients.get_mut(&id) {
            Some(c) => c,
            None => {
                panic!("PacketReaderActorReadPacketMessage should never be received by NodeActor with an unknown id {:?}", id);
            }
        };

        match (client.r#type.as_ref(), packet) {
            (None, Packet::FollowerHandshakeChallenge) => {
                let id = next_key(&mut self.followers);

                let follower = Rc::new(NodeFollower::new(id));
                if self.followers.insert(id, follower.clone()).is_some() {
                    panic!("followers::insert with id {} should never return Some", id);
                }
                client.set_type(Some(NodeClientType::Follower(follower)));

                let packet = match (Packet::FollowerHandshakeSuccess { id }).to_vec() {
                    Err(e) => {
                        eprintln!("{}", e);
                        context.stop();
                        return;
                    }
                    Ok(p) => p,
                };

                client.stream().do_send(TcpStreamActorSendMessage::new(packet.into()));
            }
            _ => {
                error!("Unhandled packet");
            }
        };
    }
}

impl Handler<NodeActorRemoveClientMessage> for NodeActor {
    type Result = <NodeActorRemoveClientMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveClientMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        trace!("Client disconnected");

        let client = self
            .clients
            .get_mut(&message.id())
            .expect("NodeActorRemoveClientMessage should never be received by NodeActor with an unknown client id");

        if let Some(NodeClientType::Follower(follower)) = client.r#type() {
            self
                .followers
                .remove(&follower.id())
                .expect("NodeActorRemoveClientMessage should never be received NodeActor with an unknown follower id");
        }

        self
            .clients
            .remove(&message.id())
            .expect("NodeActorRemoveClientMessage should never be received by NodeActor with an unknown client id");
    }
}

impl Handler<TcpListenerActorAcceptMessage> for NodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let id = next_key(&mut self.clients);

        let (stream,) = message.into();
        let (reader, stream) = PacketReaderActor::new(id, context.address(), stream.unwrap());
        self.clients.insert(id, NodeClient::new(id, reader, stream));

        trace!("Client connected");
    }
}

impl Handler<TcpStreamActorReceiveMessage> for NodeActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (buffer, _) = match self.r#type {
            NodeType::Follower {
                ref mut buffer,
                ref mut stream,
                ..
            } => (buffer, stream),
            _ => {
                panic!("TcpStreamActorReceiveMessage should be never received by NodeActor with type {:?}", self.r#type);
            }
        };

        let (result,) = message.into();

        let bytes = match result {
            Ok(b) => b,
            Err(e) => {
                eprintln!("{:?}", e);
                context.stop();
                return;
            }
        };

        buffer.extend(bytes);

        loop {
            let (read, packet) = match Packet::from_vec(&buffer) {
                Ok((r, p)) => (r, p),
                Err(ReadPacketError::UnexpectedEnd) => break,
                Err(e) => {
                    eprintln!("{:?}", e);
                    context.stop();
                    return;
                }
            };

            buffer.drain(0..read);

            trace!("Received packet");

            match packet {
                Packet::FollowerHandshakeSuccess { id } => {
                    self.id = id;
                }
                _ => {
                    panic!("Unhandled packet");
                }
            }
        }
    }
}

impl NodeActor {
    pub async fn new<S, T>(
        listener_address: S,
        stream_address: Option<T>,
    ) -> Result<Addr<Self>, io::Error>
    where
        S: ToSocketAddrs,
        T: ToSocketAddrs,
    {
        init();

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
                    r#type: NodeType::Follower {
                        stream: TcpStreamActor::new(
                            context.address().recipient(),
                            Some(context.address().recipient()),
                            stream,
                        ),
                        buffer: Vec::default(),
                    },
                    clients: HashMap::default(),
                    followers: HashMap::default(),
                }))
            }
            None => Ok(Self::create(move |context| Self {
                id: 0,
                listener: TcpListenerActor::new(
                    context.address().recipient(),
                    Some(context.address().recipient()),
                    listener,
                ),
                r#type: NodeType::Leader {},
                clients: HashMap::default(),
                followers: HashMap::default(),
            })),
        }
    }
}
