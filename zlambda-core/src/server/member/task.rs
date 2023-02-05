use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::general::{
    GeneralLogEntriesAppendRequestMessage, GeneralLogEntriesAppendRequestMessageInput,
    GeneralLogEntriesAppendResponseMessage, GeneralMessage,
};
use crate::server::member::{
    ServerMemberMessage, ServerMemberRecoveryMessage, ServerMemberRegistrationMessage,
    ServerMemberReplicationMessage,
};
use crate::server::{
    ServerId, ServerLogEntriesAcknowledgementMessageInput, ServerLogEntriesRecoveryMessageInput,
    ServerMessage,
};
use tokio::{select, spawn};
use tracing::{error, info};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerMemberTask {
    server_id: ServerId,
    server_queue_sender: MessageQueueSender<ServerMessage>,
    general_socket: Option<(
        MessageSocketSender<GeneralMessage>,
        MessageSocketReceiver<GeneralMessage>,
    )>,
    sender: MessageQueueSender<ServerMemberMessage>,
    receiver: MessageQueueReceiver<ServerMemberMessage>,
}

impl ServerMemberTask {
    pub fn new(
        server_id: ServerId,
        server_queue_sender: MessageQueueSender<ServerMessage>,
        general_socket: Option<(
            MessageSocketSender<GeneralMessage>,
            MessageSocketReceiver<GeneralMessage>,
        )>,
    ) -> Self {
        let (sender, receiver) = message_queue();

        Self {
            server_id,
            server_queue_sender,
            general_socket,
            sender,
            receiver,
        }
    }

    pub fn sender(&self) -> &MessageQueueSender<ServerMemberMessage> {
        &self.sender
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    pub async fn run(mut self) {
        loop {
            self.select().await
        }
    }

    async fn select(&mut self) {
        match &mut self.general_socket {
            Some(ref mut general_socket) => {
                select!(
                    result = general_socket.1.receive() => {
                        match result {
                            Err(error) => {
                                self.general_socket = None;
                                error!("{}", error);
                                info!("Server {} connection lost", self.server_id)
                            }
                            Ok(None) => {
                                self.general_socket = None;
                                info!("Server {} connection lost", self.server_id)
                            }
                            Ok(Some(message)) => {
                                self.on_general_message(message).await
                            }
                        }
                    }
                    message = self.receiver.do_receive() => {
                        self.on_server_member_message(message).await
                    }
                )
            }
            None => {
                select!(
                    message = self.receiver.do_receive() => {
                        self.on_server_member_message(message).await
                    }
                )
            }
        }
    }

    async fn on_server_member_message(&mut self, message: ServerMemberMessage) {
        match message {
            ServerMemberMessage::Replication(message) => {
                self.on_server_member_replication_message(message).await
            }
            ServerMemberMessage::Registration(message) => {
                self.on_server_member_registration_message(message).await
            }
            ServerMemberMessage::Recovery(message) => {
                self.on_server_member_recovery_message(message).await
            }
        }
    }

    async fn on_server_member_replication_message(
        &mut self,
        message: ServerMemberReplicationMessage,
    ) {
        match &mut self.general_socket {
            Some(ref mut general_socket) => {
                let (input,) = message.into();
                let (log_entries, last_committed_log_entry_id, log_current_term) = input.into();

                if let Err(error) = general_socket
                    .0
                    .send(GeneralLogEntriesAppendRequestMessage::new(
                        GeneralLogEntriesAppendRequestMessageInput::new(
                            log_entries,
                            last_committed_log_entry_id,
                            log_current_term,
                        ),
                    ))
                    .await
                {
                    error!("{}", error);
                    let _ = general_socket;
                    self.general_socket = None;
                }
            }
            None => {}
        }
    }

    async fn on_server_member_registration_message(
        &mut self,
        message: ServerMemberRegistrationMessage,
    ) {
        if self.general_socket.is_some() {
            panic!("Expect socket to be none");
        }

        info!("Server {} registered", self.server_id);

        let (input,) = message.into();
        let (mut sender, receiver, last_committed_log_entry_id, log_current_term) = input.into();

        if let Err(error) = sender
            .send(GeneralLogEntriesAppendRequestMessage::new(
                GeneralLogEntriesAppendRequestMessageInput::new(
                    Vec::default(),
                    last_committed_log_entry_id,
                    log_current_term,
                ),
            ))
            .await
        {
            error!("{}", error);
            return;
        }

        self.general_socket = Some((sender, receiver));
    }

    async fn on_server_member_recovery_message(&mut self, message: ServerMemberRecoveryMessage) {
        if self.general_socket.is_some() {
            panic!("Expect socket to be none");
        }

        let (input,) = message.into();
        let (mut sender, receiver, last_committed_log_entry_id, log_current_term) = input.into();

        if let Err(error) = sender
            .send(GeneralLogEntriesAppendRequestMessage::new(
                GeneralLogEntriesAppendRequestMessageInput::new(
                    Vec::default(),
                    last_committed_log_entry_id,
                    log_current_term,
                ),
            ))
            .await
        {
            error!("{}", error);
            return;
        }

        self.general_socket = Some((sender, receiver));

        info!("Server {} recovered", self.server_id);
    }

    async fn on_general_message(&mut self, message: GeneralMessage) {
        match message {
            GeneralMessage::LogEntriesAppendResponse(message) => {
                self.on_general_log_entries_append_response_message(message)
                    .await
            }
            message => {
                error!(
                    "{}",
                    MessageError::UnexpectedMessage(format!("{message:?}"))
                );
            }
        }
    }

    async fn on_general_log_entries_append_response_message(
        &mut self,
        message: GeneralLogEntriesAppendResponseMessage,
    ) {
        let (input,) = message.into();
        let (acknowledged_log_entry_ids, missing_log_entry_ids) = input.into();

        self.server_queue_sender
            .do_send_asynchronous(ServerLogEntriesAcknowledgementMessageInput::new(
                acknowledged_log_entry_ids,
                self.server_id,
            ))
            .await;

        if !missing_log_entry_ids.is_empty() {
            self.server_queue_sender
                .do_send_asynchronous(ServerLogEntriesRecoveryMessageInput::new(
                    self.server_id,
                    missing_log_entry_ids,
                ))
                .await;
        }
    }
}
