#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Args, Parser, Subcommand};
use futures::stream::StreamExt;
use std::error::Error;
use zlambda_core::client::{
    ClientModule, ClientModuleInitializeEventInput, ClientModuleInitializeEventOutput, ClientTask,
};
use zlambda_core::common::future::stream::{empty, iter};
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::common::notification::NotificationBodyItemStreamExt;
use zlambda_core::server::{
    ServerBuilder, ServerId, ServerModule, ServerModuleNotificationEventInput,
    ServerModuleNotificationEventOutput,
};
use zlambda_scheduling::round_robin::RoundRobinNotificationHeader;
use zlambda_scheduling::round_robin::RoundRobinSchedulingModule;

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
    Notify { module_id: ModuleId },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct TestClientModule {}

#[async_trait::async_trait]
impl Module for TestClientModule {}

#[async_trait::async_trait]
impl ClientModule for TestClientModule {
    async fn on_initialize(
        &self,
        event: ClientModuleInitializeEventInput,
    ) -> ClientModuleInitializeEventOutput {
        let mut iter = iter([]);

        let mut stream = iter.writer();
        stream
            .serialize(&RoundRobinNotificationHeader::new(1usize))
            .unwrap();

        event.client_handle().server().notify(0, stream).await;
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default, Debug)]
pub struct PrintServerModule {}

#[async_trait::async_trait]
impl Module for PrintServerModule {}

#[async_trait::async_trait]
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
                .add_module(RoundRobinSchedulingModule::default())
                .add_module(PrintServerModule::default())
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
                    TestClientModule::default(),
                ))]
                .into_iter(),
            )
            .await?;

            /*match command {
                /*ClientCommand::Notify { module_id } => {
                    client_task
                        .handle()
                        .server()
                        .notify(module_id, ReaderStream::new(stdin()).map(|x| x.unwrap()))
                        .await;
                }*/
            }*/

            client_task.run().await
        }
    };

    Ok(())
}
