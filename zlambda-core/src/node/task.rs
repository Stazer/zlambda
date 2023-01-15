use crate::message::{MessageQueueSender, MessageQueueReceiver, message_queue};
use crate::log::Log;
use crate::node::{NodeId, NodeInternalMessage};
use tokio::net::TcpListener;
use std::net::SocketAddr;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct NodeTask {
    node_id: NodeId,
    leader_node_id: NodeId,
    node_socket_addresses: Vec<Option<SocketAddr>>,
    log: Log,
    tcp_listener: TcpListener,
    sender: MessageQueueSender<NodeInternalMessage>,
    receiver: MessageQueueReceiver<NodeInternalMessage>,
}

impl NodeTask {

}
