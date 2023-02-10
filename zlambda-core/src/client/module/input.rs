use crate::client::ClientHandle;
use crate::common::message::MessageQueueReceiver;
use crate::common::utility::Bytes;
use async_stream::stream;
use futures::Stream;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientModuleNotificationEventInputBody {
    receiver: MessageQueueReceiver<Bytes>,
}

impl ClientModuleNotificationEventInputBody {
    pub fn new(receiver: MessageQueueReceiver<Bytes>) -> Self {
        Self { receiver }
    }

    pub fn stream(&mut self) -> impl Stream<Item = Bytes> + '_ {
        stream!(while let Some(bytes) = self.receiver.receive().await {
            yield bytes
        })
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientModuleNotificationEventInput {
    client_handle: ClientHandle,
    body: ClientModuleNotificationEventInputBody,
}

impl From<ClientModuleNotificationEventInput> for (ClientHandle, ClientModuleNotificationEventInputBody) {
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
