use crate::cluster::{
    FollowerNodeActor, NodeActor, NodeRegisterResponsePacketError, Packet, PacketReader,
    UnregisteredFollowerNodeActor, UpdateFollowerNodeActorMessage,
};
use crate::common::{
    TcpListenerActor, TcpStreamActor, TcpStreamActorReceiveMessage, TcpStreamActorSendMessage,
};
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RegisteringFollowerNodeActor {
    node_actor_address: Addr<NodeActor>,
    follower_node_actor_address: Addr<FollowerNodeActor>,
    tcp_listener_actor_address: Addr<TcpListenerActor>,
    tcp_listener_socket_local_address: SocketAddr,
    tcp_stream_actor_address: Addr<TcpStreamActor>,
    packet_reader: PacketReader,
}

impl Actor for RegisteringFollowerNodeActor {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        let bytes = match (Packet::NodeRegisterRequest {
            local_address: self.tcp_listener_socket_local_address,
        }
        .to_bytes())
        {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("{}", e);
                context.stop();

                return;
            }
        };

        match self
            .tcp_stream_actor_address
            .try_send(TcpStreamActorSendMessage::new(bytes))
        {
            Ok(()) => {}
            Err(e) => {
                error!("{}", e);
                context.stop();
            }
        }
    }
}

impl Handler<TcpStreamActorReceiveMessage> for RegisteringFollowerNodeActor {
    type Result = <TcpStreamActorReceiveMessage as Message>::Result;

    #[tracing::instrument]
    fn handle(
        &mut self,
        message: TcpStreamActorReceiveMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        let (bytes,) = message.into();

        let bytes = match bytes {
            Ok(bytes) => bytes,
            Err(error) => {
                error!("{}", error);
                context.stop();

                return;
            }
        };

        self.packet_reader.push(bytes);

        loop {
            let packet = match self.packet_reader.next() {
                Ok(Some(packet)) => packet,
                Ok(None) => {
                    break;
                }
                Err(error) => {
                    error!("{}", error);
                    context.stop();

                    return;
                }
            };

            let response = match packet {
                Packet::NodeRegisterResponse { result } => result,
                packet => {
                    error!("Unhandled packet {:?}", packet);
                    context.stop();

                    return;
                }
            };

            match response {
                Ok(_data) => {}
                Err(NodeRegisterResponsePacketError::NotALeader { leader_address }) => {
                    match self.follower_node_actor_address.try_send(
                        UpdateFollowerNodeActorMessage::new(FollowerNodeActor::Unregistered(
                            UnregisteredFollowerNodeActor::new(
                                leader_address,
                                self.node_actor_address.clone(),
                                self.follower_node_actor_address.clone(),
                                self.tcp_listener_actor_address.clone(),
                                self.tcp_listener_socket_local_address,
                            ),
                        )),
                    ) {
                        Ok(()) => {}
                        Err(e) => {
                            error!("{}", e);
                            context.stop();
                        }
                    }
                }
            }
        }
    }
}

impl RegisteringFollowerNodeActor {
    pub fn new(
        node_actor_address: Addr<NodeActor>,
        follower_node_actor_address: Addr<FollowerNodeActor>,
        tcp_listener_actor_address: Addr<TcpListenerActor>,
        tcp_listener_socket_local_address: SocketAddr,
        tcp_stream: TcpStream,
    ) -> Addr<Self> {
        Self::create(move |context| Self {
            node_actor_address,
            follower_node_actor_address,
            tcp_listener_actor_address,
            tcp_listener_socket_local_address,
            tcp_stream_actor_address: TcpStreamActor::new(
                context.address().recipient(),
                tcp_stream,
            ),
            packet_reader: PacketReader::default(),
        })
    }
}
