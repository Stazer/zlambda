use actix::{Actor, Addr, Context, StreamHandler};
use bytes::BytesMut;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{ToSocketAddrs, UdpSocket};
use tokio_util::codec::BytesCodec;
use tokio_util::udp::UdpFramed;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct UdpSocketActor {
    socket: Arc<UdpSocket>,
}

impl Actor for UdpSocketActor {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut <Self as Actor>::Context) {
        Self::add_stream(
            UdpFramed::new(self.socket.clone(), BytesCodec::new()),
            context,
        );
    }
}

impl StreamHandler<Result<(BytesMut, SocketAddr), io::Error>> for UdpSocketActor {
    fn handle(
        &mut self,
        item: Result<(BytesMut, SocketAddr), io::Error>,
        _: &mut <Self as Actor>::Context,
    ) {
        println!("{:?}", item);
    }
}

impl UdpSocketActor {
    pub async fn new<T>(socket_address: T) -> Result<Addr<Self>, io::Error>
    where
        T: ToSocketAddrs,
    {
        Ok((Self {
            socket: Arc::new(UdpSocket::bind(socket_address).await?),
        })
        .start())
    }
}
