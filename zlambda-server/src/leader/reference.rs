use tokio::sync::{mpsc, oneshot};
use crate::leader::LeaderMessage;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct InternalLeaderHandle {
    sender: mpsc::Sender<LeaderMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ExternalLeaderHandle {
    sender: mpsc::Sender<LeaderMessage>,
}

impl ExternalLeaderHandle {
    pub async fn dispatch(&self, id: ModuleId, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        let (sender, receiver) = oneshot::channel();

        self.sender.send(LeaderMessage::Dispatch);await

        Ok(receiver.await)
    }
}
