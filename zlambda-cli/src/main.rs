#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Args, Parser, Subcommand};
use std::error::Error;
use std::path::Path;
use zlambda_core::client::ClientTask;
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
use zlambda_process::ProcessDispatcher;
use zlambda_router::round_robin::{GlobalRoundRobinRouter, LocalRoundRobinRouter};
use zlambda_scheduler::real_time::DeadlineRealTimeScheduler;

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
        #[clap(default_value = "0.0.0.0:8000")]
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
pub struct PrintServerModule {}

#[async_trait]
impl Module for PrintServerModule {}

#[async_trait]
impl ServerModule for PrintServerModule {
    async fn on_notification(
        &self,
        mut input: ServerModuleNotificationEventInput,
    ) -> ServerModuleNotificationEventOutput {
        while let Some(bytes) = input
            .notification_body_item_queue_receiver_mut()
            .next()
            .await
        {
            println!("{:?}", bytes);
        }
    }
}

impl PrintServerModule {
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
                .add_module(DynamicLibraryManager::default())
                //.add_module(LocalRoundRobinRouter::default())
                .add_module(PrintServerModule::default())
                //.add_module(GlobalRoundRobinRouter::default())
                .add_module(ProcessDispatcher::default())
                .add_module(DeadlineRealTimeScheduler::default())
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
            let client_task = ClientTask::new(address, vec![].into_iter()).await?;

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
