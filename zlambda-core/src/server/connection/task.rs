use crate::general::{
    GeneralMessage, GeneralRecoveryRequestMessage, GeneralRecoveryResponseMessageInput,
    GeneralRecoveryResponseMessageNotALeaderInput, GeneralRecoveryResponseMessageSuccessInput,
    GeneralRegistrationRequestMessage, GeneralRegistrationResponseMessageInput,
    GeneralRegistrationResponseMessageNotALeaderInput,
    GeneralRegistrationResponseMessageSuccessInput,
};
use crate::message::{
    MessageError, MessageQueueSender, MessageSocketReceiver, MessageSocketSender,
};
use crate::server::member::{
    ServerMemberRecoveryMessageInput, ServerMemberRegistrationMessageInput,
};
use crate::server::{
    ServerMessage, ServerRecoveryMessageInput, ServerRecoveryMessageOutput,
    ServerRegistrationMessageInput, ServerRegistrationMessageOutput,
};
use tokio::spawn;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerConnectionTask {
    server_queue_sender: MessageQueueSender<ServerMessage>,
    general_socket_sender: MessageSocketSender<GeneralMessage>,
    general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
}

impl ServerConnectionTask {
    pub fn new(
        server_queue_sender: MessageQueueSender<ServerMessage>,
        general_socket_sender: MessageSocketSender<GeneralMessage>,
        general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        Self {
            server_queue_sender,
            general_socket_sender,
            general_socket_receiver,
        }
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        let message = match self.general_socket_receiver.receive().await {
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
                MessageError::UnexpectedMessage(format!("{message:?}"))
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
            .server_queue_sender
            .do_send_synchronous(ServerRegistrationMessageInput::new(server_socket_address))
            .await
        {
            ServerRegistrationMessageOutput::NotALeader(output) => {
                if let Err(error) = self
                    .general_socket_sender
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
                let (
                    server_id,
                    leader_server_id,
                    server_socket_addresses,
                    log_term,
                    member_queue_sender,
                ) = output.into();

                if let Err(error) = self
                    .general_socket_sender
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
                } else {
                    member_queue_sender
                        .do_send_asynchronous(ServerMemberRegistrationMessageInput::new(
                            self.general_socket_sender,
                            self.general_socket_receiver,
                        ))
                        .await;
                }
            }
        }
    }

    async fn on_general_recovery_request_message(mut self, message: GeneralRecoveryRequestMessage) {
        let (input,) = message.into();
        let (server_id,) = input.into();

        match self
            .server_queue_sender
            .do_send_synchronous(ServerRecoveryMessageInput::new(server_id))
            .await
        {
            ServerRecoveryMessageOutput::NotALeader(output) => {
                if let Err(error) = self
                    .general_socket_sender
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
            ServerRecoveryMessageOutput::IsOnline => {
                if let Err(error) = self
                    .general_socket_sender
                    .send_asynchronous(GeneralRecoveryResponseMessageInput::IsOnline)
                    .await
                {
                    error!("{}", error);
                }
            }
            ServerRecoveryMessageOutput::Unknown => {
                if let Err(error) = self
                    .general_socket_sender
                    .send_asynchronous(GeneralRecoveryResponseMessageInput::Unknown)
                    .await
                {
                    error!("{}", error);
                }
            }
            ServerRecoveryMessageOutput::Success(output) => {
                let (leader_server_id, server_socket_address, log_term, member_queue_sender) =
                    output.into();

                if let Err(error) = self
                    .general_socket_sender
                    .send_asynchronous(GeneralRecoveryResponseMessageInput::Success(
                        GeneralRecoveryResponseMessageSuccessInput::new(
                            leader_server_id,
                            server_socket_address,
                            log_term,
                        ),
                    ))
                    .await
                {
                    error!("{}", error);
                } else {
                    member_queue_sender
                        .do_send_asynchronous(ServerMemberRecoveryMessageInput::new(
                            self.general_socket_sender,
                            self.general_socket_receiver,
                        ))
                        .await;
                }
            }
        }
    }
}
