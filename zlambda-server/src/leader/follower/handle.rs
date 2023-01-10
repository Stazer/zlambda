use crate::leader::follower::{
    LeaderFollowerRecoveryMessage, LeaderFollowerMessage, LeaderFollowerReplicateMessage,
    LeaderFollowerStatus, LeaderFollowerStatusMessage,
};
use std::error::Error;
use tokio::sync::{mpsc, oneshot};
use zlambda_common::channel::{DoReceive, DoSend};
use zlambda_common::log::{LogEntryData, LogEntryId};
use zlambda_common::message::{GuestToLeaderMessageStreamReader, LeaderToGuestMessageStreamWriter};
use zlambda_common::term::Term;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
pub struct LeaderFollowerHandle {
    sender: mpsc::Sender<LeaderFollowerMessage>,
}

impl LeaderFollowerHandle {
    pub fn new(sender: mpsc::Sender<LeaderFollowerMessage>) -> Self {
        Self { sender }
    }

    pub async fn status(&self) -> LeaderFollowerStatus {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderFollowerMessage::Status(
                LeaderFollowerStatusMessage::new(sender),
            ))
            .await;

        receiver.do_receive().await
    }

    pub async fn replicate(
        &self,
        term: Term,
        last_committed_log_entry_id: Option<LogEntryId>,
        log_entry_data: Vec<LogEntryData>,
    ) {
        self.sender
            .do_send(LeaderFollowerMessage::Replicate(
                LeaderFollowerReplicateMessage::new(
                    term,
                    last_committed_log_entry_id,
                    log_entry_data,
                ),
            ))
            .await;
    }

    pub async fn recovery(
        &self,
        reader: GuestToLeaderMessageStreamReader,
        writer: LeaderToGuestMessageStreamWriter,
        term: Term,
        acknowledging_log_entry_data: Vec<LogEntryData>,
        last_committed_log_entry_id: Option<LogEntryId>,
    ) -> Result<(), Box<dyn Error>> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .do_send(LeaderFollowerMessage::Recovery(
                LeaderFollowerRecoveryMessage::new(
                    reader.into(),
                    writer.into(),
                    term,
                    acknowledging_log_entry_data,
                    last_committed_log_entry_id,
                    sender,
                ),
            ))
            .await;

        receiver.do_receive().await.expect("");

        Ok(())
    }
}
