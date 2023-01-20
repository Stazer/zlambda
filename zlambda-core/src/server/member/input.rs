use crate::server::LogEntry;
use crate::message::{MessageSocketSender, MessageSocketReceiver};
use crate::general::GeneralMessage;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerMemberReplicationMessageInput {
    log_entries: Vec<LogEntry>,
}

impl From<ServerMemberReplicationMessageInput> for (Vec<LogEntry>,) {
    fn from(input: ServerMemberReplicationMessageInput) -> Self {
        (input.log_entries,)
    }
}

impl ServerMemberReplicationMessageInput {
    pub fn new(log_entries: Vec<LogEntry>) -> Self {
        Self { log_entries }
    }

    pub fn log_entries(&self) -> &Vec<LogEntry> {
        &self.log_entries
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ServerMemberRegistrationMessageInput {
    general_socket_sender: MessageSocketSender<GeneralMessage>,
    general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
}

impl From<ServerMemberRegistrationMessageInput> for (MessageSocketSender<GeneralMessage>, MessageSocketReceiver<GeneralMessage>) {
    fn from(input: ServerMemberRegistrationMessageInput) -> Self {
        (input.general_socket_sender, input.general_socket_receiver)
    }
}

impl ServerMemberRegistrationMessageInput {
    pub fn new(
        general_socket_sender: MessageSocketSender<GeneralMessage>,
        general_socket_receiver: MessageSocketReceiver<GeneralMessage>,
    ) -> Self {
        Self {
            general_socket_sender,
            general_socket_receiver,
        }
    }

    pub fn general_socket_sender(&self) -> &MessageSocketSender<GeneralMessage> {
        &self.general_socket_sender
    }

    pub fn general_socket_receiver(&self) -> &MessageSocketReceiver<GeneralMessage> {
        &self.general_socket_receiver
    }
}
