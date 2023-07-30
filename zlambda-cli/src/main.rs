#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Args, Parser, Subcommand};
use std::error::Error;
use std::path::Path;
use zlambda_core::client::{
    ClientModule, ClientModuleNotificationEventInput, ClientModuleNotificationEventOutput,
    ClientTask,
};
use zlambda_core::common::async_trait;
use zlambda_core::common::fs::read;
use zlambda_core::common::future::stream::{empty, StreamExt};
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::common::utility::Bytes;
use zlambda_core::server::{
    ServerBuilder, ServerId, ServerModule, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventOutput,
};
use zlambda_dynamic::DynamicLibraryManager;
use zlambda_matrix_native::MatrixCalculator;
use zlambda_matrix_wasm_module::ImmediateWasmExecutor;
use zlambda_process::ProcessDispatcher;
use zlambda_realtime_task::RealTimeTaskManager;
use zlambda_router::round_robin::{GlobalRoundRobinRouter, LocalRoundRobinRouter};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct MainArguments {
    #[clap(subcommand)]
    command: MainCommand,
    #[arg(short, long, default_value = "false")]
    tokio_console: bool,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Args)]
struct FollowerData {
    address: String,
    server_id: ServerId,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum MainCommand {
    Server {
        #[clap(default_value = "0.0.0.0:8000")]
        listener_address: String,
        #[clap(subcommand)]
        command: ServerCommand,
    },
    Client {
        #[clap(default_value = "127.0.0.1:8000")]
        address: String,
        #[clap(subcommand)]
        command: ClientCommand,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum ServerCommand {
    Leader,
    Follower {
        leader_address: String,
        server_id: Option<ServerId>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum ClientCommand {
    Notify {
        module_id: ModuleId,
        bodies: Vec<String>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct PrintModule {}

#[async_trait]
impl Module for PrintModule {}

#[async_trait]
impl ClientModule for PrintModule {
    async fn on_notification(
        &self,
        mut input: ClientModuleNotificationEventInput,
    ) -> ClientModuleNotificationEventOutput {
        let mut stdout = stdout();

        while let Some(bytes) = input.body_mut().receiver_mut().next().await {
            stdout.write_all(&bytes).unwrap();
        }
    }
}

#[async_trait]
impl ServerModule for PrintModule {
    async fn on_notification(
        &self,
        mut input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        let mut stdout = stdout();

        while let Some(bytes) = input
            .notification_body_item_queue_receiver_mut()
            .next()
            .await
        {
            stdout.write_all(&bytes).unwrap();
        }
    }
}

impl PrintModule {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

use std::io::{stdout, Write};

#[derive(Default, Debug)]
pub struct PrintAndExitModule {}

#[async_trait]
impl Module for PrintAndExitModule {}

#[async_trait]
impl ClientModule for PrintAndExitModule {
    async fn on_notification(
        &self,
        mut input: ClientModuleNotificationEventInput,
    ) -> ClientModuleNotificationEventOutput {
        let mut stdout = stdout();

        while let Some(bytes) = input.body_mut().receiver_mut().next().await {
            stdout.write_all(&bytes).unwrap();
        }

        input.client_handle().exit().await;
    }
}

impl PrintAndExitModule {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let arguments = MainArguments::parse();

    match arguments.command {
        MainCommand::Server {
            listener_address,
            command,
        } => {
            if arguments.tokio_console {
                console_subscriber::init();
            } else {
                tracing_subscriber::fmt::init();
            }

            ServerBuilder::default()
                .add_module(PrintModule::default())
                .add_module(ProcessDispatcher::default())
                .add_module(RealTimeTaskManager::default())
                .add_module(GlobalRoundRobinRouter::default())
                .add_module(LocalRoundRobinRouter::default())
                .add_module(DynamicLibraryManager::default())
                .add_module(ImmediateWasmExecutor::default())
                .add_module(MatrixCalculator::default())
                //.add_module(zlambda_ebpf_loader::EbpfLoader::default())
                .build(
                    listener_address,
                    match command {
                        ServerCommand::Leader => None,
                        ServerCommand::Follower {
                            leader_address,
                            server_id,
                        } => Some((leader_address, server_id)),
                    },
                )
                .await?
                .wait()
                .await;
        }
        MainCommand::Client { address, command } => {
            let client_task = ClientTask::new(
                address,
                vec![Box::<dyn ClientModule>::from(Box::new(
                    PrintAndExitModule::default(),
                ))]
                .into_iter(),
            )
            .await?;

            // TODO

            match command {
                ClientCommand::Notify { module_id, bodies } => {
                    let mut serializer = empty().serializer();

                    for body in bodies {
                        if Path::new(&body).extension().map(|x| x.to_str()) == Some(Some("json")) {
                            serializer.serialize_json(Bytes::from(read(body).await?))?;
                        } else {
                            serializer.serialize_binary(Bytes::from(read(body).await?))?;
                        }
                    }

                    client_task
                        .handle()
                        .server()
                        .notify(module_id, serializer)
                        .await;
                }
            }

            client_task.run().await
        }
    };

    Ok(())
}
