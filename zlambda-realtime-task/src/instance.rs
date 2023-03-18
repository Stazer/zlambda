use crate::{DeadlineSortableRealTimeTask, RealTimeTask, RealTimeTaskSchedulingTaskMessage};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use zlambda_core::common::message::MessageQueueSender;
use zlambda_core::common::notification::{
    NotificationBodyItemQueueReceiver, NotificationBodyStreamDeserializer, NotificationId,
};
use zlambda_core::common::sync::RwLock;
use zlambda_core::server::LogId;
use zlambda_core::server::{Server, ServerId};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct RealTimeTaskManagerInstance {
    local_counter: AtomicUsize,
    log_id: LogId,
    receivers: RwLock<
        HashMap<
            NotificationId,
            NotificationBodyStreamDeserializer<NotificationBodyItemQueueReceiver>,
        >,
    >,
    tasks: RwLock<Vec<RealTimeTask>>,
    deadline_sorted_tasks: RwLock<BinaryHeap<Reverse<DeadlineSortableRealTimeTask>>>,
    occupations: RwLock<HashMap<ServerId, HashSet<RealTimeTask>>>,
    sender: MessageQueueSender<RealTimeTaskSchedulingTaskMessage>,
    server: Arc<Server>,
}

impl RealTimeTaskManagerInstance {
    pub fn new(
        local_counter: AtomicUsize,
        log_id: LogId,
        receivers: RwLock<
            HashMap<
                NotificationId,
                NotificationBodyStreamDeserializer<NotificationBodyItemQueueReceiver>,
            >,
        >,
        tasks: RwLock<Vec<RealTimeTask>>,
        deadline_sorted_tasks: RwLock<BinaryHeap<Reverse<DeadlineSortableRealTimeTask>>>,
        occupations: RwLock<HashMap<ServerId, HashSet<RealTimeTask>>>,
        sender: MessageQueueSender<RealTimeTaskSchedulingTaskMessage>,
        server: Arc<Server>,
    ) -> Self {
        Self {
            local_counter,
            log_id,
            receivers,
            tasks,
            deadline_sorted_tasks,
            occupations,
            sender,
            server,
        }
    }

    pub fn local_counter(&self) -> &AtomicUsize {
        &self.local_counter
    }

    pub fn log_id(&self) -> LogId {
        self.log_id
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

    pub fn tasks(&self) -> &RwLock<Vec<RealTimeTask>> {
        &self.tasks
    }

    pub fn deadline_sorted_tasks(
        &self,
    ) -> &RwLock<BinaryHeap<Reverse<DeadlineSortableRealTimeTask>>> {
        &self.deadline_sorted_tasks
    }

    pub fn occupations(&self) -> &RwLock<HashMap<ServerId, HashSet<RealTimeTask>>> {
        &self.occupations
    }

    pub fn sender(&self) -> &MessageQueueSender<RealTimeTaskSchedulingTaskMessage> {
        &self.sender
    }

    pub fn server(&self) -> &Arc<Server> {
        &self.server
    }
}
