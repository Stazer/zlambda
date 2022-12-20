use crate::leader::client::LeaderClient;
use crate::leader::follower::LeaderFollowerBuilder;
use crate::leader::LeaderMessage;
use std::error::Error;
use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientToNodeMessage, GuestToNodeMessage, LeaderToGuestMessage, Message, MessageStreamReader,
    MessageStreamWriter,
};
use zlambda_common::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct LeaderConnectionBuilder {}

impl LeaderConnectionBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(
        self,
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_sender: mpsc::Sender<LeaderMessage>,
    ) -> LeaderConnectionTask {
        LeaderConnectionTask::new(reader, writer, leader_sender)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderConnectionTask {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_sender: mpsc::Sender<LeaderMessage>,
}

impl LeaderConnectionTask {
    fn new(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_sender: mpsc::Sender<LeaderMessage>,
    ) -> Self {
        Self {
            reader,
            writer,
            leader_sender,
        }
    }

    pub fn spawn(self) {
        spawn(async move {
            self.main().await;
        });
    }

    async fn main(mut self) {
        loop {
            select!(
                read_result = self.reader.read() => {
                    let message = match read_result {
                        Ok(message) => message,
                        Err(error) => {
                            error!("{}", error);
                            break
                        }
                    };

                    match message {
                        None => continue,
                        Some(Message::GuestToNode(message)) => {
                            self.on_unregistered_to_node_message(message).await;
                            break
                        },
                        Some(Message::ClientToNode(message)) => {
                            if let Err(error) = self.register_client(message).await {
                                error!("{}", error);
                            }

                            break
                        }
                        Some(message) => {
                            error!("Unhandled message {:?}", message);
                            break
                        }
                    };
                }
            )
        }
    }

    async fn on_unregistered_to_node_message(self, message: GuestToNodeMessage) {
        match message {
            GuestToNodeMessage::RegisterRequest { address } => {
                self.register_follower(address).await.expect("");
            }
            GuestToNodeMessage::HandshakeRequest { address, node_id } => {
                self.handshake_follower(address, node_id).await.expect("");
            }
        }
    }

    async fn register_follower(self, address: SocketAddr) -> Result<(), Box<dyn Error>> {
        let (result_sender, result_receiver) = oneshot::channel();
        let builder = LeaderFollowerBuilder::new();
        let handle = builder.handle();

        self.leader_sender
            .send(LeaderMessage::Register(address, handle, result_sender))
            .await?;

        let (id, leader_id, term, addresses) = result_receiver.await?;

        let mut writer = self.writer.into();

        writer
            .write(LeaderToGuestMessage::RegisterOkResponse {
                id,
                leader_id,
                term,
                addresses,
            })
            .await?;

        builder
            .build(
                id,
                Some(self.reader.into()),
                Some(writer.into()),
                self.leader_sender,
            )
            .spawn();

        Ok(())
    }

    async fn handshake_follower(
        self,
        address: SocketAddr,
        node_id: NodeId,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    async fn register_client(
        self,
        initial_message: ClientToNodeMessage,
    ) -> Result<(), Box<dyn Error>> {
        spawn(async move {
            LeaderClient::new(
                self.reader.into(),
                self.writer.into(),
                self.leader_sender,
                initial_message,
            )
            .await
            .run()
            .await;
        });

        Ok(())
    }
}
