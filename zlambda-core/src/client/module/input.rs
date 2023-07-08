use crate::client::ClientHandle;
use crate::common::message::MessageQueueReceiver;
use crate::common::utility::Bytes;
use async_stream::stream;
use futures::Stream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientModuleInitializeEventInput {
    client_handle: ClientHandle,
}

impl From<ClientModuleInitializeEventInput> for (ClientHandle,) {
    fn from(input: ClientModuleInitializeEventInput) -> Self {
        (input.client_handle,)
    }
}

impl ClientModuleInitializeEventInput {
    pub fn new(client_handle: ClientHandle) -> Self {
        Self { client_handle }
    }

    pub fn client_handle(&self) -> &ClientHandle {
        &self.client_handle
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientModuleFinalizeEventInput {
    client_handle: ClientHandle,
}

impl From<ClientModuleFinalizeEventInput> for (ClientHandle,) {
    fn from(input: ClientModuleFinalizeEventInput) -> Self {
        (input.client_handle,)
    }
}

impl ClientModuleFinalizeEventInput {
    pub fn new(client_handle: ClientHandle) -> Self {
        Self { client_handle }
    }

    pub fn client_handle(&self) -> &ClientHandle {
        &self.client_handle
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientModuleNotificationEventInputBody {
    receiver: MessageQueueReceiver<Bytes>,
}

impl ClientModuleNotificationEventInputBody {
    pub fn new(receiver: MessageQueueReceiver<Bytes>) -> Self {
        Self { receiver }
    }

    pub fn receiver_mut(&mut self) -> &mut MessageQueueReceiver<Bytes> {
        &mut self.receiver
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientModuleNotificationEventInput {
    client_handle: ClientHandle,
    body: ClientModuleNotificationEventInputBody,
}

impl From<ClientModuleNotificationEventInput>
    for (ClientHandle, ClientModuleNotificationEventInputBody)
{
    fn from(input: ClientModuleNotificationEventInput) -> Self {
        (input.client_handle, input.body)
    }
}

impl ClientModuleNotificationEventInput {
    pub fn new(client_handle: ClientHandle, body: ClientModuleNotificationEventInputBody) -> Self {
        Self {
            client_handle,
            body,
        }
    }

    pub fn client_handle(&self) -> &ClientHandle {
        &self.client_handle
    }

    pub fn body(&self) -> &ClientModuleNotificationEventInputBody {
        &self.body
    }

    pub fn body_mut(&mut self) -> &mut ClientModuleNotificationEventInputBody {
        &mut self.body
    }
}
