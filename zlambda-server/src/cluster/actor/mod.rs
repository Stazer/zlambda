mod leader;
mod node;
mod packet_reader;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub use node::*;
pub use packet_reader::*;

////////////////////////////////////////////////////////////////////////////////////////////////////

use crate::common::{ActorStopMessage, TcpStreamActor};
use actix::{Actor, ActorContext, Addr, Context, Handler, Message};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ReplicatorActor {
    node_actor_address: Addr<NodeActor>,
    stream_actor_address: Addr<TcpStreamActor>,
}

impl Actor for ReplicatorActor {
    type Context = Context<Self>;
}

impl Handler<ActorStopMessage> for ReplicatorActor {
    type Result = <ActorStopMessage as Message>::Result;

    fn handle(
        &mut self,
        _message: ActorStopMessage,
        context: &mut <Self as Actor>::Context,
    ) -> Self::Result {
        context.stop();
    }
}
