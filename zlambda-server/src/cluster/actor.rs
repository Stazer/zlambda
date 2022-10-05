use crate::cluster::{
    ConnectionId, NodeActorRegisterMessage, NodeActorRemoveConnectionMessage, NodeState, Packet,
    PacketReaderActor, PacketReaderActorReadPacketMessage,
};
use crate::common::{
    ActorExecuteMessage, ActorStopMessage, TcpListenerActor, TcpListenerActorAcceptMessage,
    TcpStreamActor, TcpStreamActorSendMessage,
};
use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message,
    ResponseActFuture, WrapFuture,
};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tracing::{error, trace};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct NodeActor {
    listener_address: Addr<TcpListenerActor>,
    stream_addresses: HashMap<ConnectionId, Addr<TcpStreamActor>>,
    reader_addresses: HashMap<ConnectionId, Addr<PacketReaderActor>>,
    state: NodeState,
}

impl Actor for NodeActor {
    type Context = Context<Self>;

    fn stopped(&mut self, _context: &mut Self::Context) {
        self.listener_address.do_send(ActorStopMessage);

        for stream in self.stream_addresses.values() {
            stream.do_send(ActorStopMessage);
        }

        for reader in self.reader_addresses.values() {
            reader.do_send(ActorStopMessage);
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

impl Handler<TcpListenerActorAcceptMessage> for NodeActor {
    type Result = <TcpListenerActorAcceptMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpListenerActorAcceptMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        trace!("Create connection");

        let stream = match message.into() {
            (Err(e),) => {
                error!("{}", e);
                return;
            }
            (Ok(s),) => s,
        };

        let connection_id = self.state.create_connection();

        let (reader_address, stream_address) =
            PacketReaderActor::new(connection_id, context.address(), stream);
        self.stream_addresses.insert(connection_id, stream_address);
        self.reader_addresses.insert(connection_id, reader_address);

        trace!("Connection {} created", connection_id);
    }
}

impl Handler<NodeActorRemoveConnectionMessage> for NodeActor {
    type Result = <NodeActorRemoveConnectionMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRemoveConnectionMessage,
        _context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        trace!("Destroy connection {}", message.connection_id());

        self.stream_addresses.remove(&message.connection_id());
        self.reader_addresses.remove(&message.connection_id());
        self.state.destroy_connection(message.connection_id());

        trace!("Connection destroyed");
    }
}

impl<F> Handler<ActorExecuteMessage<F, NodeActor>> for NodeActor
where
    F: FnOnce(&mut NodeActor, &mut <NodeActor as Actor>::Context),
{
    type Result = <ActorExecuteMessage<F, NodeActor> as Message>::Result;

    fn handle(
        &mut self,
        message: ActorExecuteMessage<F, NodeActor>,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (function,) = message.into();
        function(self, context);
    }
}

impl Handler<PacketReaderActorReadPacketMessage> for NodeActor {
    //type Result = ResponseActFuture<Self, <PacketReaderActorReadPacketMessage as Message>::Result>;
    type Result = <PacketReaderActorReadPacketMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: PacketReaderActorReadPacketMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (connection_id, packet): (ConnectionId, Packet) = message.into();
        let stream_address = self
            .reader_addresses
            .get(&connection_id)
            .expect("Reader not found")
            .clone();

        match packet {
            Packet::NodeHandshakeRequest { node_id } => {
                self.state.handle_node_handshake_request(node_id);
            }
            Packet::NodeHandshakeResponse { result } => {
                trace!("{:?}", result);
            }
            Packet::NodeRegisterRequest => {
                trace!("request");
            }
            _ => {
                context.wait(
                    async move { stream_address.send(ActorStopMessage).await }
                        .into_actor(self)
                        .map(|result, _actor, context| {
                            if let Err(e) = result {
                                error!("{}", e);
                                context.stop();
                            }
                        })
                );
            }
        };
    }
}

impl Handler<NodeActorRegisterMessage> for NodeActor {
    type Result = ResponseActFuture<Self, <NodeActorRegisterMessage as Message>::Result>;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: NodeActorRegisterMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (stream,): (TcpStream,) = message.into();

        let connection_id = self.state.create_connection();

        let (reader_address, stream_address) =
            PacketReaderActor::new(connection_id, context.address(), stream);
        self.stream_addresses
            .insert(connection_id, stream_address.clone());
        self.reader_addresses.insert(connection_id, reader_address);

        async move {
            let packet = match Packet::NodeRegisterRequest.to_bytes() {
                Err(e) => return Err(Box::<dyn Error>::from(e)),
                Ok(p) => p,
            };

            match stream_address
                .send(TcpStreamActorSendMessage::new(packet))
                .await
            {
                Err(e) => Err(Box::<dyn Error>::from(e)),
                Ok(Err(e)) => Err(Box::<dyn Error>::from(e)),
                Ok(Ok(())) => Ok(()),
            }
        }
        .into_actor(self)
        .map(|result, _actor, context| {
            if let Err(error) = result {
                error!("{}", error);
                context.stop();
            }
        })
        .boxed_local()
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
        let stream = match stream_address {
            Some(s) => Some(TcpStream::connect(s).await?),
            None => None,
        };

        let listener = TcpListener::bind(listener_address).await?;

        let actor = Self::create(move |context| Self {
            listener_address: TcpListenerActor::new(
                context.address().recipient(),
                Some(context.address().recipient()),
                listener,
            ),
            reader_addresses: HashMap::default(),
            stream_addresses: HashMap::default(),
            state: NodeState::default(),
        });

        if let Some(stream) = stream {
            actor
                .send(NodeActorRegisterMessage::new(stream))
                .await
                .expect("Cannot send NodeActorRegisterMessage");
        }

        Ok(actor)
    }
}
