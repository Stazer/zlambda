#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use byteorder::{ByteOrder, LittleEndian};
use clap::{Args, Parser, Subcommand};
use std::error::Error;
use std::mem::size_of;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use zlambda_core::client::{
    ClientModule, ClientModuleNotificationEventInput, ClientModuleNotificationEventOutput,
    ClientTask, ClientModuleInitializeEventInput, ClientModuleInitializeEventOutput,
};
use zlambda_core::common::async_trait;
use zlambda_core::common::fs::read;
use zlambda_core::common::future::stream::{empty, StreamExt};
use zlambda_core::common::io::{stdin, stdout, AsyncReadExt, AsyncWriteExt};
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::net::UdpSocket;
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::common::process::Command;
use zlambda_core::common::runtime::spawn;
use zlambda_core::common::utility::Bytes;
use zlambda_core::server::{
    ServerBuilder, ServerId, ServerModule, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventOutput,
};
use zlambda_dynamic::DynamicLibraryManager;
use zlambda_matrix::MATRIX_SIZE;
use zlambda_matrix_ebpf_module::EbpfLoader;
use zlambda_matrix_native::MatrixCalculator;
use zlambda_matrix_wasm_module::ImmediateWasmExecutor;
use zlambda_process::ProcessDispatcher;
use zlambda_realtime_task::RealTimeTaskManager;
use zlambda_sleep::SleepModule;
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
        #[arg(short, long, default_value = "false")]
        exit: bool,
    },
    Ebpf {
        #[clap(default_value = "127.0.0.1:10200")]
        address: String,
    },
    Matrix {},
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
            stdout.write_all(&bytes).await.unwrap();
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
            stdout.write_all(&bytes).await.unwrap();
        }
    }
}

impl PrintModule {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct ExitModule {}

#[async_trait]
impl Module for ExitModule {}

#[async_trait]
impl ClientModule for ExitModule {
    async fn on_initialize(
        &self,
        input: ClientModuleInitializeEventInput,
    ) -> ClientModuleInitializeEventOutput {
        input.client_handle().exit().await;
    }
}

impl ExitModule {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

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
            stdout.write_all(&bytes).await.unwrap();
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
    let program_begin = Instant::now();

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
                .add_module(EbpfLoader::default())
                .add_module(SleepModule::default())
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
        MainCommand::Client { address, command, exit } => {
            let mut modules = Vec::new();

            if exit {
                modules.push(Box::<dyn ClientModule>::from(Box::new(
                    ExitModule::default(),
                )));
            } else {
                modules.push(Box::<dyn ClientModule>::from(Box::new(
                    PrintAndExitModule::default(),
                )));
            }

            let client_task = ClientTask::new(
                address,
                modules
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

            client_task.run().await;

            if exit {
                return Ok(());
            }

            let mut buffer: [u8; size_of::<u128>()] = [0; size_of::<u128>()];
            LittleEndian::write_u128(&mut buffer, program_begin.elapsed().as_nanos());
            stdout().write_all(&buffer).await?;
        }
        MainCommand::Ebpf { address } => {
            let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
            socket.connect(address).await?;
            let receiver = socket.clone();
            let sender = socket;

            spawn(async move {
                let mut buffer: [u8; 3 * MATRIX_SIZE + 2 * size_of::<u128>()] =
                    [0; 3 * MATRIX_SIZE + 2 * size_of::<u128>()];
                stdin().read(&mut buffer).await.expect("Success");
                sender.send(&buffer).await.expect("Success");
            });

            let mut buffer: [u8; 3 * MATRIX_SIZE + 3 * size_of::<u128>()] =
                [0; 3 * MATRIX_SIZE + 3 * size_of::<u128>()];
            receiver.recv(&mut buffer).await?;
            LittleEndian::write_u128(
                &mut buffer[3 * MATRIX_SIZE + 2 * size_of::<u128>()..],
                program_begin.elapsed().as_nanos(),
            );
            stdout().write_all(&buffer).await?;
        }
        MainCommand::Matrix {} => {
            Command::new("target/release/zlambda-matrix-process")
                .spawn()?
                .wait()
                .await?;

            let mut buffer: [u8; size_of::<u128>()] = [0; size_of::<u128>()];
            LittleEndian::write_u128(&mut buffer, program_begin.elapsed().as_nanos());
            let mut stdout = stdout();
            stdout.write_all(&buffer).await?;
            stdout.flush().await?;
        }
    };

    Ok(())
}
