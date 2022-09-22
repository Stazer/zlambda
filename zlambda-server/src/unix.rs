use actix::{Actor, Addr, Context, StreamHandler};
use std::io::Error;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task;
use std::task::Poll;
use tokio::net::unix::SocketAddr;
use tokio::net::{UnixListener, UnixStream};
use tokio_stream::Stream;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct UnixListenerActor {
    listener: Arc<UnixListener>,
}

impl Actor for UnixListenerActor {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut <Self as Actor>::Context) {
        struct Streamer {
            listener: Arc<UnixListener>,
        }

        impl Stream for Streamer {
            type Item = Result<(UnixStream, SocketAddr), Error>;

            fn poll_next(
                self: Pin<&mut Self>,
                cx: &mut task::Context<'_>,
            ) -> Poll<Option<Result<(UnixStream, SocketAddr), Error>>> {
                match self.listener.poll_accept(cx) {
                    Poll::Ready(Ok((stream, address))) => Poll::Ready(Some(Ok((stream, address)))),
                    Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
                    Poll::Pending => Poll::Pending,
                }
            }
        }

        Self::add_stream(
            Streamer {
                listener: self.listener.clone(),
            },
            context,
        );
    }
}

impl StreamHandler<Result<(UnixStream, SocketAddr), Error>> for UnixListenerActor {
    fn handle(
        &mut self,
        item: Result<(UnixStream, SocketAddr), Error>,
        _: &mut <Self as Actor>::Context,
    ) {
        println!("{:?}", item);
    }
}

impl UnixListenerActor {
    pub async fn new<T>(path: T) -> Result<Addr<Self>, Error>
    where
        T: AsRef<Path>,
    {
        Ok((Self {
            listener: Arc::new(UnixListener::bind(path)?),
        })
        .start())
    }
}
