use crate::client::NewClientError;
use crate::common::message::{MessageError, MessageSocketReceiver, MessageSocketSender};
use crate::common::runtime::spawn;
use crate::common::net::{TcpStream, ToSocketAddrs};
use crate::general::{
    GeneralClientRegistrationRequestMessage, GeneralClientRegistrationRequestMessageInput,
    GeneralMessage,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ClientTask {
    sender: MessageSocketSender<GeneralMessage>,
    receiver: MessageSocketReceiver<GeneralMessage>,
}

impl ClientTask {
    pub async fn new<T>(address: T) -> Result<Self, NewClientError>
    where
        T: ToSocketAddrs,
    {
        let socket = TcpStream::connect(address).await?;

        let (reader, writer) = socket.into_split();

        let (mut sender, mut receiver) = (
            MessageSocketSender::<GeneralMessage>::new(writer),
            MessageSocketReceiver::<GeneralMessage>::new(reader),
        );

        sender
            .send(GeneralClientRegistrationRequestMessage::new(
                GeneralClientRegistrationRequestMessageInput,
            ))
            .await?;

        match receiver.receive().await? {
            None => return Err(MessageError::ExpectedMessage.into()),
            Some(GeneralMessage::ClientRegistrationResponse(_)) => {}
            Some(message) => {
                return Err(MessageError::UnexpectedMessage(format!("{message:?}")).into())
            }
        }

        Ok(Self { sender, receiver })
    }

    pub fn spawn(self) {
        spawn(async move {
            self.run().await
        });
    }

    pub async fn run(mut self) {
        loop {
            self.select().await
        }
    }

    async fn select(&mut self) {

    }
}
