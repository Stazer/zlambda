use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::general::{
    GeneralLogEntriesAppendRequestMessage, GeneralLogEntriesAppendRequestMessageInput,
    GeneralLogEntriesAppendResponseMessage, GeneralMessage, GeneralNotifyMessage,
    GeneralNotifyMessageInput,
};
use crate::server::node::{
    ServerNodeMessage, ServerNodeNotifyMessage, ServerNodeRecoveryMessage,
    ServerNodeRegistrationMessage, ServerNodeReplicationMessage,
};
use crate::server::{
    ServerId, ServerLogEntriesAcknowledgementMessageInput, ServerLogEntriesRecoveryMessageInput,
    ServerMessage, ServerNotifyMessageInput, ServerNotifyMessageInputServerSource,
};
use tokio::{select, spawn};
use tracing::{error, info};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerNodeTask {
    server_id: ServerId,
    server_queue_sender: MessageQueueSender<ServerMessage>,
    general_socket: Option<(
        MessageSocketSender<GeneralMessage>,
        MessageSocketReceiver<GeneralMessage>,
    )>,
    sender: MessageQueueSender<ServerNodeMessage>,
    receiver: MessageQueueReceiver<ServerNodeMessage>,
}

impl ServerNodeTask {
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

    pub fn sender(&self) -> &MessageQueueSender<ServerNodeMessage> {
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
                        self.on_server_node_message(message).await
                    }
                )
            }
            None => {
                select!(
                    message = self.receiver.do_receive() => {
                        self.on_server_node_message(message).await
                    }
                )
            }
        }
    }

    async fn on_server_node_message(&mut self, message: ServerNodeMessage) {
        match message {
            ServerNodeMessage::Replication(message) => {
                self.on_server_node_replication_message(message).await
            }
            ServerNodeMessage::Registration(message) => {
                self.on_server_node_registration_message(message).await
            }
            ServerNodeMessage::Recovery(message) => {
                self.on_server_node_recovery_message(message).await
            }
            ServerNodeMessage::Notify(message) => self.on_server_node_notify_message(message).await,
        }
    }

    async fn on_server_node_replication_message(&mut self, message: ServerNodeReplicationMessage) {
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

    async fn on_server_node_registration_message(
        &mut self,
        message: ServerNodeRegistrationMessage,
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

    async fn on_server_node_recovery_message(&mut self, message: ServerNodeRecoveryMessage) {
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

    async fn on_server_node_notify_message(&mut self, message: ServerNodeNotifyMessage) {
        let socket = match &mut self.general_socket {
            None => return,
            Some(socket) => socket,
        };

        let (input,) = message.into();
        let (module_id, body) = input.into();

        if let Err(error) = socket
            .0
            .send(GeneralNotifyMessage::new(GeneralNotifyMessageInput::new(
                module_id, body,
            )))
            .await
        {
            error!("{}", error);
            return;
        }
    }

    async fn on_general_message(&mut self, message: GeneralMessage) {
        match message {
            GeneralMessage::LogEntriesAppendResponse(message) => {
                self.on_general_log_entries_append_response_message(message)
                    .await
            }
            GeneralMessage::Notify(message) => self.on_general_notify_message(message).await,
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

    async fn on_general_notify_message(&mut self, message: GeneralNotifyMessage) {
        let (input,) = message.into();
        let (module_id, body) = input.into();

        self.server_queue_sender
            .do_send_asynchronous(ServerNotifyMessageInput::new(
                module_id,
                ServerNotifyMessageInputServerSource::new(self.server_id).into(),
                body,
            ))
            .await;
    }
}
