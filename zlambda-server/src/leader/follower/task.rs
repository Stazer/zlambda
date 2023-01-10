use crate::leader::follower::{
    LeaderFollowerRecoveryMessage, LeaderFollowerMessage, LeaderFollowerReplicateMessage,
    LeaderFollowerResult, LeaderFollowerStatus, LeaderFollowerStatusMessage,
};
use crate::leader::LeaderHandle;
use tokio::sync::mpsc;
use tokio::{select, spawn};
use tracing::{error, info};
use zlambda_common::channel::DoSend;
use zlambda_common::message::{
    FollowerToLeaderAppendEntriesResponseMessage, FollowerToLeaderDispatchRequestMessage,
    FollowerToLeaderMessage, FollowerToLeaderMessageStreamReader,
    LeaderToFollowerAppendEntriesRequestMessage, LeaderToFollowerMessage,
    LeaderToFollowerMessageStreamWriter, LeaderToGuestRecoveryOkResponseMessage,
    LeaderToGuestMessage,
};
use zlambda_common::node::NodeId;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct LeaderFollowerTask {
    id: NodeId,
    receiver: mpsc::Receiver<LeaderFollowerMessage>,
    reader: Option<FollowerToLeaderMessageStreamReader>,
    writer: Option<LeaderToFollowerMessageStreamWriter>,
    leader_handle: LeaderHandle,
}

impl LeaderFollowerTask {
    pub fn new(
        id: NodeId,
        receiver: mpsc::Receiver<LeaderFollowerMessage>,
        reader: Option<FollowerToLeaderMessageStreamReader>,
        writer: Option<LeaderToFollowerMessageStreamWriter>,
        leader_handle: LeaderHandle,
    ) -> Self {
        Self {
            id,
            receiver,
            reader,
            writer,
            leader_handle,
        }
    }

    pub fn spawn(self) {
        spawn(async move { self.run().await });
    }

    async fn run(mut self) {
        self.on_initialize().await;

        loop {
            self.select().await
            /*{
                LeaderFollowerResult::Continue => continue,
                LeaderFollowerResult::ConnectionClosed => {
                    self.writer = None;
                    self.reader = None;
                }*
            }*/
        }
    }

    async fn select(&mut self) {
        match self.reader {
            Some(ref mut reader) => {
                select!(
                    result = reader.read() => {
                        match result {
                            Ok(None) => {
                                info!("Node {} disconnected", self.id);
                                self.reader = None;
                                self.writer = None;
                            }
                            Ok(Some(message)) => {
                                self.on_registered_follower_to_leader_message(message).await;
                            }
                            Err(error) => {
                                error!("{}", error);
                            }
                        }
                    }
                    result = self.receiver.recv() => {
                        if let Some(message) = result {
                            self.on_leader_follower_message(message).await;
                        }
                    }
                );
            }
            None => {
                select!(
                    result = self.receiver.recv() => {
                        if let Some(message) = result {
                            self.on_leader_follower_message(message).await;
                        }
                    }
                );
            }
        };
    }

    async fn on_initialize(&mut self) -> LeaderFollowerResult {
        info!("Node {} registered", self.id);

        if let Some(ref mut writer) = &mut self.writer {
            let status = self.leader_handle.replication_status().await;

            if writer
                .write(LeaderToFollowerMessage::AppendEntriesRequest(
                    LeaderToFollowerAppendEntriesRequestMessage::new(
                        status.term(),
                        *status.last_committed_log_entry_id(),
                        Vec::default(),
                    ),
                ))
                .await
                .is_err()
            {
                return LeaderFollowerResult::ConnectionClosed;
            }
        }

        LeaderFollowerResult::Continue
    }

    async fn on_registered_follower_to_leader_message(&mut self, message: FollowerToLeaderMessage) {
        match message {
            FollowerToLeaderMessage::AppendEntriesResponse(message) => {
                self.on_follower_to_leader_append_entries_response_message(message)
                    .await
            }
            FollowerToLeaderMessage::DispatchRequest(message) => {
                self.on_follower_to_leader_dispatch_request_message(message)
                    .await
            }
        }
    }

    async fn on_follower_to_leader_append_entries_response_message(
        &mut self,
        message: FollowerToLeaderAppendEntriesResponseMessage,
    ) {
        let (appended_log_entry_ids, missing_log_entry_ids) = message.into();

        let missing_log_entry_data = self
            .leader_handle
            .acknowledge(self.id, appended_log_entry_ids, missing_log_entry_ids)
            .await;

        if missing_log_entry_data.is_empty() {
            return;
        }

        let writer = match &mut self.writer {
            Some(ref mut writer) => writer,
            None => return,
        };

        let status = self.leader_handle.replication_status().await;

        writer
            .write(LeaderToFollowerMessage::AppendEntriesRequest(
                LeaderToFollowerAppendEntriesRequestMessage::new(
                    status.term(),
                    *status.last_committed_log_entry_id(),
                    missing_log_entry_data,
                ),
            ))
            .await
            .expect("");
    }

    async fn on_follower_to_leader_dispatch_request_message(
        &mut self,
        _message: FollowerToLeaderDispatchRequestMessage,
    ) {
    }

    async fn on_leader_follower_message(
        &mut self,
        message: LeaderFollowerMessage,
    ) -> LeaderFollowerResult {
        match message {
            LeaderFollowerMessage::Replicate(message) => {
                self.on_leader_follower_replicate_message(message).await
            }
            LeaderFollowerMessage::Recovery(message) => {
                self.on_leader_follower_recovery_message(message).await
            }
            LeaderFollowerMessage::Status(message) => {
                self.on_leader_follower_status_message(message).await
            }
        }
    }

    async fn on_leader_follower_recovery_message(
        &mut self,
        message: LeaderFollowerRecoveryMessage,
    ) -> LeaderFollowerResult {
        let (
            reader,
            mut writer,
            term,
            last_committed_log_entry_id,
            acknowledging_log_entry_data,
            sender,
        ) = message.into();

        if self.reader.is_none() && self.writer.is_none() {
            if writer
                .write(LeaderToGuestMessage::RecoveryOkResponse(
                    LeaderToGuestRecoveryOkResponseMessage::new(0),
                ))
                .await
                .is_err()
            {
                return LeaderFollowerResult::ConnectionClosed;
            }

            let mut writer = writer.into();

            if writer
                .write(LeaderToFollowerMessage::AppendEntriesRequest(
                    LeaderToFollowerAppendEntriesRequestMessage::new(
                        term,
                        acknowledging_log_entry_data,
                        last_committed_log_entry_id,
                    ),
                ))
                .await
                .is_err()
            {
                return LeaderFollowerResult::ConnectionClosed;
            }

            self.reader = Some(reader.into());
            self.writer = Some(writer);

            sender.do_send(Ok(())).await;

            info!("Node {} recovered", self.id);
        } else {
            sender.do_send(Err("Follower is online".into())).await;
        }

        LeaderFollowerResult::Continue
    }

    async fn on_leader_follower_status_message(
        &mut self,
        message: LeaderFollowerStatusMessage,
    ) -> LeaderFollowerResult {
        let (sender,) = message.into();

        sender
            .do_send(LeaderFollowerStatus::new(
                self.reader.is_some() && self.writer.is_some(),
            ))
            .await;

        LeaderFollowerResult::Continue
    }

    async fn on_leader_follower_replicate_message(
        &mut self,
        message: LeaderFollowerReplicateMessage,
    ) -> LeaderFollowerResult {
        let (term, last_committed_log_entry_id, log_entry_data) = message.into();
        let mut result = LeaderFollowerResult::Continue;

        if let Some(ref mut writer) = &mut self.writer {
            if writer
                .write(LeaderToFollowerMessage::AppendEntriesRequest(
                    LeaderToFollowerAppendEntriesRequestMessage::new(
                        term,
                        last_committed_log_entry_id,
                        log_entry_data,
                    ),
                ))
                .await
                .is_err()
            {
                result = LeaderFollowerResult::ConnectionClosed;
            }
        }

        result
    }
}
