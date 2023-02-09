use crate::client::ClientHandle;
use crate::common::utility::Bytes;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct ClientModuleNotifyEventInput {
    client_handle: ClientHandle,
    body: Bytes,
}

impl From<ClientModuleNotifyEventInput> for (ClientHandle, Bytes) {
    fn from(input: ClientModuleNotifyEventInput) -> Self {
        (input.client_handle, input.body)
    }
}

impl ClientModuleNotifyEventInput {
    pub fn new(client_handle: ClientHandle, body: Bytes) -> Self {
        Self {
            client_handle,
            body,
        }
    }

    pub fn client_handle(&self) -> &ClientHandle {
        &self.client_handle
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }
}
