use crate::common::message::{
    message_queue, MessageError, MessageQueueReceiver, MessageQueueSender, MessageSocketReceiver,
    MessageSocketSender,
};
use crate::common::runtime::{select, spawn};
use crate::common::utility::Bytes;
use crate::general::{
    GeneralLogEntriesAppendRequestMessage, GeneralLogEntriesAppendRequestMessageInput,
    GeneralLogEntriesAppendResponseMessage, GeneralLogEntriesAppendResponseMessageInput,
    GeneralMessage, GeneralNotificationMessage, GeneralNotificationMessageInput,
    GeneralNotificationMessageInputEndType, GeneralNotificationMessageInputImmediateType,
    GeneralNotificationMessageInputNextType, GeneralNotificationMessageInputStartType,
    GeneralNotificationMessageInputType,
};
use crate::server::node::{
    ServerNodeLogAppendResponseMessage, ServerNodeMessage, ServerNodeNotificationEndMessage,
    ServerNodeNotificationImmediateMessage, ServerNodeNotificationNextMessage,
    ServerNodeNotificationStartMessage, ServerNodeNotificationStartMessageOutput,
    ServerNodeRecoveryMessage, ServerNodeRegistrationMessage,
    ServerNodeReplicationMessage,
};
use crate::server::{
    ServerHandle, ServerId, ServerLogAppendRequestMessageInput,
    ServerLogEntriesAcknowledgementMessageInput, ServerLogEntriesRecoveryMessageInput,
    ServerMessage, ServerModuleGetMessageInput, ServerModuleNotificationEventBody,
    ServerModuleNotificationEventInput, ServerModuleNotificationEventInputServerSource,
};
use std::collections::HashMap;
use tracing::{error, info};

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ServerNodeTask {
    server_id: ServerId,
    server_message_sender: MessageQueueSender<ServerMessage>,
    general_socket: Option<(
        MessageSocketSender<GeneralMessage>,
        MessageSocketReceiver<GeneralMessage>,
    )>,
    sender: MessageQueueSender<ServerNodeMessage>,
    receiver: MessageQueueReceiver<ServerNodeMessage>,
    incoming_notification_senders: HashMap<usize, MessageQueueSender<Bytes>>,
    outgoing_notification_counter: usize,
}

impl ServerNodeTask {
    pub fn new(
        server_id: ServerId,
        server_message_sender: MessageQueueSender<ServerMessage>,
        general_socket: Option<(
            MessageSocketSender<GeneralMessage>,
            MessageSocketReceiver<GeneralMessage>,
        )>,
    ) -> Self {
        let (sender, receiver) = message_queue();

        Self {
            server_id,
            server_message_sender,
            general_socket,
            sender,
            receiver,
            incoming_notification_senders: HashMap::default(),
            outgoing_notification_counter: 0,
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
            ServerNodeMessage::LogAppendResponse(message) => {
                self.on_server_log_append_response_message(message).await
            }
            ServerNodeMessage::NotificationImmediate(message) => {
                self.on_server_node_notification_immediate_message(message)
                    .await
            }
            ServerNodeMessage::NotificationStart(message) => {
                self.on_server_node_notification_start_message(message)
                    .await
            }
            ServerNodeMessage::NotificationNext(message) => {
                self.on_server_node_notification_next_message(message).await
            }
            ServerNodeMessage::NotificationEnd(message) => {
                self.on_server_node_notification_end_message(message).await
            }
        }
    }

    async fn on_server_node_notification_immediate_message(
        &mut self,
        message: ServerNodeNotificationImmediateMessage,
    ) {
        let general_message_sender = match &mut self.general_socket {
            Some((sender, _)) => sender,
            None => return,
        };

        let (input,) = message.into();
        let (module_id, body) = input.into();

        if let Err(error) = general_message_sender
            .send(GeneralNotificationMessage::new(
                GeneralNotificationMessageInput::new(
                    GeneralNotificationMessageInputImmediateType::new(module_id).into(),
                    body,
                ),
            ))
            .await
        {
            error!("{}", error);
        }
    }

    async fn on_server_node_notification_start_message(
        &mut self,
        message: ServerNodeNotificationStartMessage,
    ) {
        let general_message_sender = match &mut self.general_socket {
            Some((sender, _)) => sender,
            None => return,
        };

        let (input, sender) = message.into();
        let (module_id, body) = input.into();

        let notification_id = self.outgoing_notification_counter;
        self.outgoing_notification_counter += 1;

        sender
            .do_send(ServerNodeNotificationStartMessageOutput::new(
                notification_id,
            ))
            .await;

        if let Err(error) = general_message_sender
            .send(GeneralNotificationMessage::new(
                GeneralNotificationMessageInput::new(
                    GeneralNotificationMessageInputStartType::new(module_id, notification_id)
                        .into(),
                    body,
                ),
            ))
            .await
        {
            error!("{}", error);
        }
    }

    async fn on_server_node_notification_next_message(
        &mut self,
        message: ServerNodeNotificationNextMessage,
    ) {
        let general_message_sender = match &mut self.general_socket {
            Some((sender, _)) => sender,
            None => return,
        };

        let (input,) = message.into();
        let (notification_id, body) = input.into();

        if let Err(error) = general_message_sender
            .send(GeneralNotificationMessage::new(
                GeneralNotificationMessageInput::new(
                    GeneralNotificationMessageInputNextType::new(notification_id).into(),
                    body,
                ),
            ))
            .await
        {
            error!("{}", error);
        }
    }

    async fn on_server_node_notification_end_message(
        &mut self,
        message: ServerNodeNotificationEndMessage,
    ) {
        let general_message_sender = match &mut self.general_socket {
            Some((sender, _)) => sender,
            None => return,
        };

        let (input,) = message.into();
        let (notification_id, body) = input.into();

        if let Err(error) = general_message_sender
            .send(GeneralNotificationMessage::new(
                GeneralNotificationMessageInput::new(
                    GeneralNotificationMessageInputEndType::new(notification_id).into(),
                    body,
                ),
            ))
            .await
        {
            error!("{}", error);
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

    async fn on_server_log_append_response_message(
        &mut self,
        message: ServerNodeLogAppendResponseMessage,
    ) {
        let (input,) = message.into();
        let (log_entry_ids, missing_log_entry_ids) = input.into();

        if let Some(general_socket) = &mut self.general_socket {
            if let Err(error) = general_socket
                .0
                .send(GeneralLogEntriesAppendResponseMessage::new(
                    GeneralLogEntriesAppendResponseMessageInput::new(
                        log_entry_ids,
                        missing_log_entry_ids,
                    ),
                ))
                .await
            {
                error!("{}", error);
                return;
            }
        }
    }

    async fn on_general_message(&mut self, message: GeneralMessage) {
        match message {
            GeneralMessage::LogEntriesAppendRequest(message) => {
                self.on_general_log_entries_append_request_message(message)
                    .await
            }
            GeneralMessage::LogEntriesAppendResponse(message) => {
                self.on_general_log_entries_append_response_message(message)
                    .await
            }
            GeneralMessage::Notification(message) => {
                self.on_general_notification_message(message).await
            }
            message => {
                error!(
                    "{}",
                    MessageError::UnexpectedMessage(format!("{message:?}"))
                );
            }
        }
    }

    async fn on_general_log_entries_append_request_message(
        &mut self,
        message: GeneralLogEntriesAppendRequestMessage,
    ) {
        let (input,) = message.into();
        let (log_entries, last_committed_log_entry_id, log_current_term) = input.into();

        self.server_message_sender
            .do_send_asynchronous(ServerLogAppendRequestMessageInput::new(
                self.server_id,
                log_entries,
                last_committed_log_entry_id,
                log_current_term,
            ))
            .await;
    }

    async fn on_general_log_entries_append_response_message(
        &mut self,
        message: GeneralLogEntriesAppendResponseMessage,
    ) {
        let (input,) = message.into();
        let (acknowledged_log_entry_ids, missing_log_entry_ids) = input.into();

        self.server_message_sender
            .do_send_asynchronous(ServerLogEntriesAcknowledgementMessageInput::new(
                acknowledged_log_entry_ids,
                self.server_id,
            ))
            .await;

        if !missing_log_entry_ids.is_empty() {
            self.server_message_sender
                .do_send_asynchronous(ServerLogEntriesRecoveryMessageInput::new(
                    self.server_id,
                    missing_log_entry_ids,
                ))
                .await;
        }
    }

    async fn on_general_notification_message(&mut self, message: GeneralNotificationMessage) {
        let (input,) = message.into();
        let (r#type, body) = input.into();

        match r#type {
            GeneralNotificationMessageInputType::Immediate(r#type) => {
                let output = self
                    .server_message_sender
                    .do_send_synchronous(ServerModuleGetMessageInput::new(r#type.module_id()))
                    .await;

                let module = match output.into() {
                    (None,) => return,
                    (Some(module),) => module,
                };

                let handle = ServerHandle::new(self.server_message_sender.clone());
                let server_source =
                    ServerModuleNotificationEventInputServerSource::new(self.server_id);

                let (sender, receiver) = message_queue();
                sender.do_send(body).await;

                spawn(async move {
                    module
                        .on_notification(ServerModuleNotificationEventInput::new(
                            handle,
                            server_source.into(),
                            ServerModuleNotificationEventBody::new(receiver),
                        ))
                        .await;
                });
            }
            GeneralNotificationMessageInputType::Start(r#type) => {
                let output = self
                    .server_message_sender
                    .do_send_synchronous(ServerModuleGetMessageInput::new(r#type.module_id()))
                    .await;

                let module = match output.into() {
                    (None,) => return,
                    (Some(module),) => module,
                };

                let (sender, receiver) = message_queue();
                sender.do_send(body).await;
                self.incoming_notification_senders
                    .insert(r#type.notification_id(), sender);

                let handle = ServerHandle::new(self.server_message_sender.clone());
                let server_source =
                    ServerModuleNotificationEventInputServerSource::new(self.server_id);

                spawn(async move {
                    module
                        .on_notification(ServerModuleNotificationEventInput::new(
                            handle,
                            server_source.into(),
                            ServerModuleNotificationEventBody::new(receiver),
                        ))
                        .await;
                });
            }
            GeneralNotificationMessageInputType::Next(r#type) => {
                if let Some(sender) = self
                    .incoming_notification_senders
                    .get(&r#type.notification_id())
                {
                    sender.do_send(body).await;
                }
            }
            GeneralNotificationMessageInputType::End(r#type) => {
                if let Some(sender) = self
                    .incoming_notification_senders
                    .remove(&r#type.notification_id())
                {
                    sender.do_send(body).await;
                }
            }
        };
    }
}
