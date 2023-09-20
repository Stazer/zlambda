#[cfg(not(target_os = "linux"))]
mod not_linux {
    use zlambda_core::common::async_trait;
    use zlambda_core::common::module::Module;
    use zlambda_core::server::ServerModule;

    #[derive(Default, Debug)]
    pub struct EbpfLoader;

    #[async_trait]
    impl Module for EbpfLoader {}

    #[async_trait]
    impl ServerModule for EbpfLoader {}
}

#[cfg(target_os = "linux")]
mod linux {
    use aya::programs::{Xdp, XdpFlags};
    use aya::Bpf;
    use aya_log::BpfLogger;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::future::pending;
    use zlambda_core::common::async_trait;
    use zlambda_core::common::bytes::Bytes;
    use zlambda_core::common::deserialize::deserialize_from_bytes;
    use zlambda_core::common::module::Module;
    use zlambda_core::common::net::UdpSocket;
    use zlambda_core::common::notification::NotificationBodyItemStreamExt;
    use zlambda_core::common::runtime::spawn;
    use zlambda_core::common::sync::RwLock;
    use zlambda_core::common::task::JoinHandle;
    use zlambda_core::common::tracing::error;
    use zlambda_core::server::{
        LogId, LogIssuer, LogModuleIssuer, ServerId, ServerModule, ServerModuleCommitEventInput,
        ServerModuleCommitEventOutput, ServerModuleNotificationEventInput,
        ServerModuleNotificationEventOutput, ServerModuleStartupEventInput,
        ServerModuleStartupEventOutput, ServerSystemLogEntryData, SERVER_SYSTEM_LOG_ID,
    };
    use zlambda_matrix_ebpf::EBPF_UDP_PORT;

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Debug, Deserialize, Serialize)]
    enum LogEntryData {
        Load(Bytes),
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    struct BpfTask {
        bpf: Bpf,
    }

    impl BpfTask {
        fn new(bpf: Bpf) -> Self {
            Self { bpf }
        }

        fn spawn(mut self) -> JoinHandle<()> {
            spawn(async move {
                let _socket = UdpSocket::bind(&format!("0.0.0.0:{}", EBPF_UDP_PORT))
                    .await
                    .expect("");

                if let Err(error) = BpfLogger::init(&mut self.bpf) {
                    error!("BpfLogger::init {:?}", error);
                    return;
                }

                let program = match self.bpf.program_mut("xdp_main") {
                    Some(program) => program,
                    None => {
                        error!("Binary does not include a main program");
                        return;
                    }
                };

                let xdp: &mut Xdp = match program.try_into() {
                    Ok(xdp) => xdp,
                    Err(error) => {
                        error!("program::try_into {:?}", error);
                        return;
                    }
                };

                if let Err(error) = xdp.load() {
                    error!("xdp.load {:?}", error);
                    return;
                }

                if let Err(error) = xdp.attach("enp0s5", XdpFlags::SKB_MODE) {
                    error!("xdp.attach {:?}", error);
                }

                pending().await
            })
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Debug)]
    struct Instance {
        log_id: LogId,
        _bpf_tasks: Vec<JoinHandle<()>>,
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////

    #[derive(Debug, Deserialize, Serialize)]
    struct NotificationHeader(Bytes);

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
                    instances.insert(
                        server_id,
                        Instance {
                            log_id,
                            _bpf_tasks: Vec::default(),
                        },
                    );
                }
            }
        }

        async fn on_notification(
            &self,
            input: ServerModuleNotificationEventInput,
        ) -> ServerModuleNotificationEventOutput {
            let server_id = input.server().server_id().await;

            let log_id = {
                let instances = self.instances.read().await;
                match instances.get(&server_id).map(|instance| instance.log_id) {
                    Some(log_id) => log_id,
                    None => unreachable!(),
                }
            };

            let (server, _source, notification_body_item_queue_receiver) = input.into();

            let mut deserializer = notification_body_item_queue_receiver.deserializer();
            let data = deserializer.deserialize::<Bytes>().await.unwrap();

            server.logs().get(log_id).entries().commit(data).await;
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
                        instances.insert(
                            input.server().server_id().await,
                            Instance {
                                log_id: data.log_id(),
                                _bpf_tasks: Vec::default(),
                            },
                        );
                    }
                }
            } else {
                let server_id = input.server().server_id().await;

                let log_id = {
                    let instances = self.instances.read().await;
                    match instances.get(&server_id).map(|instance| instance.log_id) {
                        Some(log_id) => log_id,
                        None => unreachable!(),
                    }
                };

                if input.log_id() == log_id {
                    let log_entry = match input
                        .server()
                        .logs()
                        .get(log_id)
                        .entries()
                        .get(input.log_entry_id())
                        .await
                    {
                        Some(log_entry) => log_entry,
                        None => unreachable!(),
                    };

                    let bpf = match Bpf::load(log_entry.data()) {
                        Ok(bpf) => bpf,
                        Err(error) => {
                            error!("{}", error);
                            return;
                        }
                    };

                    BpfTask::new(bpf).spawn();
                }
            }
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(not(target_os = "linux"))]
pub use not_linux::*;
