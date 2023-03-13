use crate::{
    DeadlineDispatchedSortableRealTimeTask, RealTimeTask, RealTimeTaskSchedulingTaskMessage,
};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use zlambda_core::common::message::MessageQueueSender;
use zlambda_core::common::notification::{
    NotificationBodyItemQueueReceiver, NotificationBodyStreamDeserializer, NotificationId,
};
use zlambda_core::common::sync::RwLock;
use zlambda_core::server::Server;
use zlambda_core::server::{LogId, ServerId};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct RealTimeSharedData {
    local_counter: AtomicUsize,
    log_ids: RwLock<HashMap<ServerId, LogId>>,
    receivers: RwLock<
        HashMap<
            NotificationId,
            NotificationBodyStreamDeserializer<NotificationBodyItemQueueReceiver>,
        >,
    >,
    tasks: RwLock<Vec<Arc<RealTimeTask>>>,
    deadline_dispatched_sorted_tasks:
        Arc<RwLock<BinaryHeap<Reverse<DeadlineDispatchedSortableRealTimeTask>>>>,
    senders: RwLock<HashMap<ServerId, MessageQueueSender<RealTimeTaskSchedulingTaskMessage>>>,
    servers: RwLock<HashMap<ServerId, Arc<Server>>>,
}

impl RealTimeSharedData {
    pub fn local_counter(&self) -> &AtomicUsize {
        &self.local_counter
    }

    pub fn log_ids(&self) -> &RwLock<HashMap<ServerId, LogId>> {
        &self.log_ids
    }

    pub fn receivers(
        &self,
    ) -> &RwLock<
        HashMap<
            NotificationId,
            NotificationBodyStreamDeserializer<NotificationBodyItemQueueReceiver>,
        >,
    > {
        &self.receivers
    }

    pub fn tasks(&self) -> &RwLock<Vec<Arc<RealTimeTask>>> {
        &self.tasks
    }

    pub fn deadline_dispatched_sorted_tasks(
        &self,
    ) -> &Arc<RwLock<BinaryHeap<Reverse<DeadlineDispatchedSortableRealTimeTask>>>> {
        &self.deadline_dispatched_sorted_tasks
    }

    pub fn senders(
        &self,
    ) -> &RwLock<HashMap<ServerId, MessageQueueSender<RealTimeTaskSchedulingTaskMessage>>> {
        &self.senders
    }

    pub fn servers(&self) -> &RwLock<HashMap<ServerId, Arc<Server>>> {
        &self.servers
    }
}
