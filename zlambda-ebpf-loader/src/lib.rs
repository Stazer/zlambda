use zlambda_core::common::async_trait;
use zlambda_core::common::bytes::Bytes;
use std::collections::HashMap;
use zlambda_core::common::module::{Module};
use zlambda_core::common::sync::{RwLock};
use zlambda_core::common::deserialize::{deserialize_from_bytes};
use serde::{Deserialize, Serialize};
use zlambda_core::server::{
    ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput,
    ServerModuleCommitEventInput, ServerModuleCommitEventOutput,
    ServerModuleStartupEventInput, ServerModuleStartupEventOutput,
    LogModuleIssuer, ServerId, LogId,
    SERVER_SYSTEM_LOG_ID, LogIssuer, ServerSystemLogEntryData,
};
use aya::Bpf;
use aya::programs::Xdp;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
enum LogEntryData {
    Load(Bytes),
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
struct NotificationHeader {

}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct Instance {
    log_id: LogId,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct EbpfLoader {
    instances: RwLock<HashMap<ServerId, Instance>>,
}

#[async_trait]
impl Module for EbpfLoader {}

#[async_trait]
impl ServerModule for EbpfLoader {
    async fn on_startup(
        &self,
        input: ServerModuleStartupEventInput,
    ) -> ServerModuleStartupEventOutput {
        let server_id = input.server().server_id().await;
        let leader_server_id = input.server().leader_server_id().await;

        if server_id == leader_server_id {
            let log_id = input
                .server()
                .logs()
                .create(Some(LogModuleIssuer::new(input.module_id()).into()))
                .await;

            {
                let mut instances = self.instances.write().await;
                instances.insert(server_id, Instance {
                    log_id,
                });
            }
        }
    }

    async fn on_notification(
        &self,
        _input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        println!("{:?}", _input);
    }

    async fn on_commit(
        &self,
        input: ServerModuleCommitEventInput,
    ) -> ServerModuleCommitEventOutput {
        if input.log_id() == SERVER_SYSTEM_LOG_ID {
            let log_entry = input
                .server()
                .logs()
                .get(input.log_id())
                .entries()
                .get(input.log_entry_id())
                .await
                .expect("");

            let data = deserialize_from_bytes::<ServerSystemLogEntryData>(log_entry.data())
                .expect("")
                .0;

            if let ServerSystemLogEntryData::CreateLog(data) = data {
                let _issuer = LogIssuer::Module(LogModuleIssuer::new(input.module_id()));
                let server_id = input.server().server_id().await;
                let leader_server_id = input.server().leader_server_id().await;

                if matches!(data.log_issuer(), Some(_issuer)) && server_id != leader_server_id {
                    let mut instances = self.instances.write().await;
                    instances.insert(input.server().server_id().await, Instance {
                        log_id: data.log_id(),
                    });
                }
            }
        }
    }
}
