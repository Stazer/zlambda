use crate::common::message::{
    MessageError, MessageQueueSender, MessageSocketReceiver, MessageSocketSender,
};
use crate::general::{
    GeneralClientRegistrationRequestMessage, GeneralMessage, GeneralNodeHandshakeRequestMessage,
    GeneralNodeHandshakeResponseMessage, GeneralNodeHandshakeResponseMessageInput,
    GeneralNodeHandshakeResponseMessageInputResult, GeneralRecoveryRequestMessage,
    GeneralRecoveryResponseMessageInput, GeneralRecoveryResponseMessageNotALeaderInput,
    GeneralRecoveryResponseMessageSuccessInput, GeneralRegistrationRequestMessage,
    GeneralRegistrationResponseMessageInput, GeneralRegistrationResponseMessageNotALeaderInput,
    GeneralRegistrationResponseMessageSuccessInput,
};
use crate::server::node::{
    ServerNodeNodeHandshakeMessageInput, ServerNodeRecoveryMessageInput,
    ServerNodeRegistrationMessageInput,
};
use crate::server::{
    ServerClientRegistrationMessageInput, ServerMessage, ServerRecoveryMessageInput,
    ServerRecoveryMessageOutput, ServerRegistrationMessageInput, ServerRegistrationMessageOutput,
    ServerServerNodeMessageSenderGetMessageInput,
};
use tokio::spawn;
use tracing::error;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerConnectionTask {
    server_message_sender: MessageQueueSender<ServerMessage>,
    general_message_sender: MessageSocketSender<GeneralMessage>,
    general_message_receiver: MessageSocketReceiver<GeneralMessage>,
}

impl ServerConnectionTask {
    pub fn new(
        server_message_sender: MessageQueueSender<ServerMessage>,
        general_message_sender: MessageSocketSender<GeneralMessage>,
        general_message_receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        Self {
            server_message_sender,
            general_message_sender,
            general_message_receiver,
        }
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        let message = match self.general_message_receiver.receive().await {
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
            GeneralMessage::NodeHandshakeRequest(message) => {
                self.on_general_node_handshake_request_message(message)
                    .await
            }
            GeneralMessage::ClientRegistrationRequest(message) => {
                self.on_general_client_registration_request_message(message)
                    .await
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
            .server_message_sender
            .do_send_synchronous(ServerRegistrationMessageInput::new(server_socket_address))
            .await
        {
            ServerRegistrationMessageOutput::NotALeader(output) => {
                if let Err(error) = self
                    .general_message_sender
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
                    last_committed_log_entry_id,
                    current_log_term,
                    node_queue_sender,
                ) = output.into();

                if let Err(error) = self
                    .general_message_sender
                    .send_asynchronous(GeneralRegistrationResponseMessageInput::Success(
                        GeneralRegistrationResponseMessageSuccessInput::new(
                            server_id,
                            leader_server_id,
                            server_socket_addresses,
                            current_log_term,
                        ),
                    ))
                    .await
                {
                    error!("{}", error);
                } else {
                    node_queue_sender
                        .do_send_asynchronous(ServerNodeRegistrationMessageInput::new(
                            self.general_message_sender,
                            self.general_message_receiver,
                            last_committed_log_entry_id,
                            current_log_term,
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
            .server_message_sender
            .do_send_synchronous(ServerRecoveryMessageInput::new(server_id))
            .await
        {
            ServerRecoveryMessageOutput::NotALeader(output) => {
                if let Err(error) = self
                    .general_message_sender
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
                    .general_message_sender
                    .send_asynchronous(GeneralRecoveryResponseMessageInput::IsOnline)
                    .await
                {
                    error!("{}", error);
                }
            }
            ServerRecoveryMessageOutput::Unknown => {
                if let Err(error) = self
                    .general_message_sender
                    .send_asynchronous(GeneralRecoveryResponseMessageInput::Unknown)
                    .await
                {
                    error!("{}", error);
                }
            }
            ServerRecoveryMessageOutput::Success(output) => {
                let (
                    leader_server_id,
                    server_socket_address,
                    last_committed_log_entry_id,
                    current_log_term,
                    node_queue_sender,
                ) = output.into();

                if let Err(error) = self
                    .general_message_sender
                    .send_asynchronous(GeneralRecoveryResponseMessageInput::Success(
                        GeneralRecoveryResponseMessageSuccessInput::new(
                            leader_server_id,
                            server_socket_address,
                            current_log_term,
                        ),
                    ))
                    .await
                {
                    error!("{}", error);
                } else {
                    node_queue_sender
                        .do_send_asynchronous(ServerNodeRecoveryMessageInput::new(
                            self.general_message_sender,
                            self.general_message_receiver,
                            last_committed_log_entry_id,
                            current_log_term,
                        ))
                        .await;
                }
            }
        }
    }

    async fn on_general_node_handshake_request_message(
        mut self,
        message: GeneralNodeHandshakeRequestMessage,
    ) {
        let (input,) = message.into();

        let output = self
            .server_message_sender
            .do_send_synchronous(ServerServerNodeMessageSenderGetMessageInput::new(
                input.server_id(),
            ))
            .await;

        let server_node_message_sender = match output.into() {
            (Some(server_node_message_sender),) => server_node_message_sender,
            (None,) => {
                if self
                    .general_message_sender
                    .send(GeneralNodeHandshakeResponseMessage::new(
                        GeneralNodeHandshakeResponseMessageInput::new(
                            GeneralNodeHandshakeResponseMessageInputResult::Unknown,
                        ),
                    ))
                    .await
                    .is_err()
                {
                    return;
                }

                return;
            }
        };

        server_node_message_sender
            .do_send_asynchronous(ServerNodeNodeHandshakeMessageInput::new(
                self.general_message_sender,
                self.general_message_receiver,
            ))
            .await;
    }

    async fn on_general_client_registration_request_message(
        self,
        _message: GeneralClientRegistrationRequestMessage,
    ) {
        self.server_message_sender
            .do_send_asynchronous(ServerClientRegistrationMessageInput::new(
                self.general_message_sender,
                self.general_message_receiver,
            ))
            .await;
    }
}
