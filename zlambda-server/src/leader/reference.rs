use tokio::sync::{mpsc, oneshot};
use crate::leader::LeaderMessage;

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct InternalLeaderReference {
    sender: mpsc::Sender<LeaderMessage>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ExternalLeaderReference {
    sender: mpsc::Sender<LeaderMessage>,
}

impl ExternalLeaderReference {
    pub async fn dispatch(&self, id: ModuleId, payload: Vec<u8>) -> Result<Vec<u8>, String> {
        let (sender, receiver) = oneshot::channel();

        self.sender.send(LeaderMessage::Dispatch);await

        Ok(receiver.await)
    }
}
