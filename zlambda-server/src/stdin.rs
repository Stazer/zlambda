use actix::{Actor, Context, Addr, StreamHandler};
use tokio::io::{stdin};
use tokio_util::io::ReaderStream;
use bytes::Bytes;
use std::io::Error;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct StdinActor {
}

impl Actor for StdinActor {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut <Self as Actor>::Context) {
        Self::add_stream(
            ReaderStream::new(stdin()),
            context,
        );
    }
}

impl StreamHandler<Result<Bytes, Error>> for StdinActor {
    fn handle(
        &mut self,
        item: Result<Bytes, Error>,
        _: &mut <Self as Actor>::Context,
    ) {
        println!("{:?}", item);
    }
}

impl StdinActor {
    pub fn new() -> Addr<Self> {
        (Self {
        }).start()
    }
}
