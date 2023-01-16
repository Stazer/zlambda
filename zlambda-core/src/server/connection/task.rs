use crate::general::{
    GeneralMessage, GeneralRecoveryRequestMessage, GeneralRegistrationRequestMessage,
    GeneralRegistrationResponseMessageInput, GeneralRegistrationResponseMessageNotALeaderInput,
    GeneralRegistrationResponseMessageSuccessInput, GeneralRecoveryResponseMessageInput,
    GeneralRecoveryResponseMessageNotALeaderInput, GeneralRecoveryResponseMessageSuccessInput,
};
use crate::message::{
    MessageError, MessageQueueSender, MessageSocketReceiver, MessageSocketSender,
};
use crate::server::{
    ServerMessage, ServerRegistrationMessageInput, ServerRegistrationMessageOutput, ServerRecoveryMessageInput,
    ServerRecoveryMessageOutput,
};
use tokio::spawn;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerConnectionTask {
    server_sender: MessageQueueSender<ServerMessage>,
    general_sender: MessageSocketSender<GeneralMessage>,
    general_receiver: MessageSocketReceiver<GeneralMessage>,
}

impl ServerConnectionTask {
    pub fn new(
        server_sender: MessageQueueSender<ServerMessage>,
        general_sender: MessageSocketSender<GeneralMessage>,
        general_receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        Self {
            server_sender,
            general_sender,
            general_receiver,
        }
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        let message = match self.general_receiver.receive().await {
            Err(error) => {
                error!("{}", error);
                return;
            }
            Ok(None) => return,
            Ok(Some(message)) => message,
        };

        self.on_general_message(message).await;
    }

    async fn on_general_message(self, message: GeneralMessage) {
        match message {
            GeneralMessage::RegistrationRequest(message) => {
                self.on_general_registration_request_message(message).await
            }
            GeneralMessage::RecoveryRequest(message) => {
                self.on_general_recovery_request_message(message).await
            }
            message => error!(
                "{}",
                MessageError::UnexpectedMessage(format!("{:?}", message))
            ),
        }
    }

    async fn on_general_registration_request_message(
        mut self,
        message: GeneralRegistrationRequestMessage,
    ) {
        let (input,) = message.into();
        let (server_socket_address,) = input.into();

        match self
            .server_sender
            .do_send_synchronous(ServerRegistrationMessageInput::new(server_socket_address))
            .await
        {
            ServerRegistrationMessageOutput::NotALeader(output) => {
                if let Err(error) = self
                    .general_sender
                    .send_asynchronous(GeneralRegistrationResponseMessageInput::NotALeader(
                        GeneralRegistrationResponseMessageNotALeaderInput::new(
                            *output.leader_server_socket_address(),
                        ),
                    ))
                    .await
                {
                    error!("{}", error);
                }
            }
            ServerRegistrationMessageOutput::Success(output) => {
                let (server_id, leader_server_id, server_socket_addresses, log_term) =
                    output.into();

                if let Err(error) = self
                    .general_sender
                    .send_asynchronous(GeneralRegistrationResponseMessageInput::Success(
                        GeneralRegistrationResponseMessageSuccessInput::new(
                            server_id,
                            leader_server_id,
                            server_socket_addresses,
                            log_term,
                        ),
                    ))
                    .await
                {
                    error!("{}", error);
                }
            }
        }
    }

    async fn on_general_recovery_request_message(mut self, message: GeneralRecoveryRequestMessage) {
        let (input,) = message.into();
        let (server_id,) = input.into();

        match self
            .server_sender
            .do_send_synchronous(ServerRecoveryMessageInput::new(server_id))
            .await
        {
            ServerRecoveryMessageOutput::NotALeader(output) => {
                if let Err(error) = self
                    .general_sender
                    .send_asynchronous(GeneralRecoveryResponseMessageInput::NotALeader(
                        GeneralRecoveryResponseMessageNotALeaderInput::new(
                            *output.leader_server_socket_address(),
                        ),
                    ))
                    .await
                {
                    error!("{}", error);
                }
            }
            ServerRecoveryMessageOutput::Success(output) => {
                let (leader_server_id, server_socket_address, log_term) = output.into();

                if let Err(error) = self
                    .general_sender
                    .send_asynchronous(GeneralRecoveryResponseMessageInput::Success(
                        GeneralRecoveryResponseMessageSuccessInput::new(
                            leader_server_id,
                            server_socket_address,
                            log_term,
                        ).into(),
                    ))
                    .await
                {
                    error!("{}", error);
                }
            }
            ServerRecoveryMessageOutput::IsOnline => {
                if let Err(error) = self
                    .general_sender
                    .send_asynchronous(GeneralRecoveryResponseMessageInput::IsOnline)
                    .await
                {
                    error!("{}", error);
                }
            }
        }
    }
}
