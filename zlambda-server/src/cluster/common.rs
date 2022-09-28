use crate::cluster::PacketReaderActor;
use crate::common::TcpStreamActor;
use actix::Addr;
use std::rc::Rc;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub type NodeClientId = u64;
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
