use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use zlambda_core::common::async_trait;
use zlambda_core::common::deserialize::deserialize_from_bytes;
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::{
    NotificationBodyItemQueueReceiver, NotificationBodyItemStreamExt, NotificationId,
};
use zlambda_core::common::serialize::serialize_to_bytes;
use zlambda_core::common::sync::{Mutex, RwLock};
use zlambda_core::server::{
    LogId, ServerClientId, ServerId, ServerModule, ServerModuleCommitEventInput,
    ServerModuleCommitEventOutput, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventInputSource, ServerModuleNotificationEventOutput,
    ServerModuleStartupEventInput, ServerModuleStartupEventOutput, ServerNotificationOrigin,
};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalRoundRobinRouterNotificationHeader {
    module_id: ModuleId,
}

impl From<GlobalRoundRobinRouterNotificationHeader> for (ModuleId,) {
    fn from(envelope: GlobalRoundRobinRouterNotificationHeader) -> Self {
        (envelope.module_id,)
    }
}

impl GlobalRoundRobinRouterNotificationHeader {
    pub fn new(module_id: ModuleId) -> Self {
        Self { module_id }
    }

    pub fn module_id(&self) -> ModuleId {
        self.module_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalRoundRobinLogEntryOriginData {
    server_id: ServerId,
    server_client_id: ServerClientId,
}

impl GlobalRoundRobinLogEntryOriginData {
    pub fn new(server_id: ServerId, server_client_id: ServerClientId) -> Self {
        Self {
            server_id,
            server_client_id,
        }
    }

    pub fn server_id(&self) -> ServerId {
        self.server_id
    }

    pub fn server_client_id(&self) -> ServerClientId {
        self.server_client_id
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalRoundRobinLogEntryData {
    issuer_server_id: ServerId,
    local_counter: usize,
    origin: Option<GlobalRoundRobinLogEntryOriginData>,
}

impl GlobalRoundRobinLogEntryData {
    pub fn new(
        issuer_server_id: ServerId,
        local_counter: usize,
        origin: Option<GlobalRoundRobinLogEntryOriginData>,
    ) -> Self {
        Self {
            issuer_server_id,
            local_counter,
            origin,
        }
    }

    pub fn issuer_server_id(&self) -> ServerId {
        self.issuer_server_id
    }

    pub fn local_counter(&self) -> usize {
        self.local_counter
    }

    pub fn origin(&self) -> &Option<GlobalRoundRobinLogEntryOriginData> {
        &self.origin
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct GlobalRoundRobinRouter {
    global_counter: AtomicUsize,
    local_counter: AtomicUsize,
    receivers: Mutex<HashMap<NotificationId, NotificationBodyItemQueueReceiver>>,
    log_ids: RwLock<HashMap<ServerId, LogId>>,
}

#[async_trait]
impl Module for GlobalRoundRobinRouter {}

#[async_trait]
impl ServerModule for GlobalRoundRobinRouter {
    async fn on_startup(
        &self,
        input: ServerModuleStartupEventInput,
    ) -> ServerModuleStartupEventOutput {
        let server_id = input.server().server_id().await;

        if server_id == input.server().leader_server_id().await {
            let log_id = input.server().logs().create(None).await;

            let mut log_ids = self.log_ids.write().await;
            log_ids.insert(server_id, log_id);
        }
    }

    async fn on_notification(
        &self,
        input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let (server, source, notification_body_item_queue_receiver) = input.into();

        let server_id = server.server_id().await;

        let log_id = {
            let log_ids = self.log_ids.read().await;
            match log_ids.get(&server_id) {
                Some(log_id) => *log_id,
                None => return,
            }
        };

        let local_counter = self.local_counter.fetch_add(1, Ordering::Relaxed);

        {
            let mut receivers = self.receivers.lock().await;
            receivers.insert(
                NotificationId::from(local_counter),
                notification_body_item_queue_receiver,
            );
        }

        let origin = match source {
            ServerModuleNotificationEventInputSource::Server(server) => {
                if let Some(origin) = server.origin() {
                    Some(ServerNotificationOrigin::new(
                        origin.server_id(),
                        origin.server_client_id(),
                    ))
                } else {
                    None
                }
            }
            ServerModuleNotificationEventInputSource::Client(client) => Some(
                ServerNotificationOrigin::new(server.server_id().await, client.server_client_id()),
            ),
        };

        server
            .logs()
            .get(log_id)
            .entries()
            .commit(
                serialize_to_bytes(&GlobalRoundRobinLogEntryData::new(
                    server_id,
                    local_counter,
                    origin.map(|o| {
                        GlobalRoundRobinLogEntryOriginData::new(o.server_id(), o.server_client_id())
                    }),
                ))
                .expect("")
                .into(),
            )
            .await;
    }

    async fn on_commit(
        &self,
        input: ServerModuleCommitEventInput,
    ) -> ServerModuleCommitEventOutput {
        let server_id = input.server().server_id().await;

        let log_id = {
            let log_ids = self.log_ids.read().await;
            match log_ids.get(&server_id) {
                Some(log_id) => *log_id,
                None => return,
            }
        };

        if input.log_id() != log_id {
            return;
        }

        let log_entry = input
            .server()
            .logs()
            .get(log_id)
            .entries()
            .get(input.log_entry_id())
            .await
            .expect("existing log entry");

        let log_entry_data =
            deserialize_from_bytes::<GlobalRoundRobinLogEntryData>(log_entry.data())
                .expect("valid log entry data")
                .0;

        let global_counter = self.global_counter.fetch_add(1, Ordering::Relaxed);

        let next_server_id = {
            let socket_addresses = input.server().servers().socket_addresses().await;

            let mut next_server_id = global_counter % socket_addresses.len();

            loop {
                if let Some(Some(_)) = socket_addresses.get(next_server_id) {
                    break;
                }

                next_server_id += 1;

                if next_server_id >= socket_addresses.len() {
                    next_server_id = 0;
                }
            }

            ServerId::from(next_server_id)
        };

        let receiver = {
            let mut receivers = self.receivers.lock().await;
            match receivers.remove(&NotificationId::from(log_entry_data.local_counter())) {
                Some(receiver) => receiver,
                None => return,
            }
        };

        let mut deserializer = receiver.deserializer();
        let header = deserializer
            .deserialize::<GlobalRoundRobinRouterNotificationHeader>()
            .await
            .unwrap();

        if let Some(server) = input.server().servers().get(next_server_id).await {
            server
                .notify(
                    header.module_id(),
                    deserializer,
                    log_entry_data.origin().as_ref().map(|o| {
                        ServerNotificationOrigin::new(o.server_id(), o.server_client_id())
                    }),
                )
                .await;
        }
    }
}
