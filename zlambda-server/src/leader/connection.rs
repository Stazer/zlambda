use crate::leader::client::LeaderClient;
use crate::leader::follower::LeaderFollower;
use crate::leader::LeaderMessage;
use std::error::Error;
use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};
use tokio::{select, spawn};
use tracing::error;
use zlambda_common::message::{
    ClientToNodeMessage, ClusterMessageRegisterResponse, LeaderToUnregisteredFollowerMessage,
    Message, MessageStreamReader, MessageStreamWriter, UnregisteredFollowerToLeaderMessage,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderConnection {
    reader: MessageStreamReader,
    writer: MessageStreamWriter,
    leader_sender: mpsc::Sender<LeaderMessage>,
}

impl LeaderConnection {
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

    pub fn spawn(
        reader: MessageStreamReader,
        writer: MessageStreamWriter,
        leader_sender: mpsc::Sender<LeaderMessage>,
    ) {
        spawn(async move {
            Self::new(reader, writer, leader_sender).main().await;
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
                        Some(Message::UnregisteredFollowerToLeader(message)) => {
                            self.on_unregistered_follower_to_leader_message(message).await;
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

    async fn on_unregistered_follower_to_leader_message(
        self,
        message: UnregisteredFollowerToLeaderMessage,
    ) {
        match message {
            UnregisteredFollowerToLeaderMessage::RegisterRequest { address } => {
                self.register_follower(address).await.expect("");
            }
        }
    }

    async fn register_follower(self, address: SocketAddr) -> Result<(), Box<dyn Error>> {
        let (follower_sender, follower_receiver) = mpsc::channel(16);
        let (result_sender, result_receiver) = oneshot::channel();

        self.leader_sender
            .send(LeaderMessage::Register(
                address,
                follower_sender,
                result_sender,
            ))
            .await?;

        let (id, leader_id, term, addresses) = result_receiver.await?;

        let mut writer = self.writer.into();

        writer
            .write(LeaderToUnregisteredFollowerMessage::RegisterResponse(
                ClusterMessageRegisterResponse::Ok {
                    id,
                    leader_id,
                    term,
                    addresses,
                },
            ))
            .await?;

        spawn(async move {
            LeaderFollower::new(
                id,
                follower_receiver,
                self.reader.into(),
                writer.into(),
                self.leader_sender,
            )
            .run()
            .await;
        });

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
