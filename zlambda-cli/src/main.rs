#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Args, Parser, Subcommand};
use futures::stream::StreamExt;
use std::error::Error;
use std::iter::empty;
use tokio::io::stdin;
use tokio_util::io::ReaderStream;
use zlambda_core::client::{
    ClientModuleInitializeEventInput, ClientModuleInitializeEventOutput,
    ClientModule, ClientTask,
};
use zlambda_core::common::module::{Module, ModuleId};
use zlambda_core::server::{ServerId, ServerModule, ServerModuleNotificationEventInput, ServerModuleNotificationEventOutput, ServerTask};

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

use zlambda_core::common::utility::Bytes;
use futures::pin_mut;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct TestClientModule {}

#[async_trait::async_trait]
impl Module for TestClientModule {}

#[async_trait::async_trait]
impl ClientModule for TestClientModule {
    async fn on_initialize(&self, event: ClientModuleInitializeEventInput) -> ClientModuleInitializeEventOutput {
        let s = async_stream::stream! {
            while let Some(bytes) = ReaderStream::new(stdin()).next().await {
                yield bytes.unwrap();
            }
        };

        pin_mut!(s);

        event.client_handle().server().notify(0, s).await;
    }
}

impl TestClientModule {
    pub fn new() -> Self {
        Self {}
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct TestServerModule {}

#[async_trait::async_trait]
impl Module for TestServerModule {}

#[async_trait::async_trait]
impl ServerModule for TestServerModule {
    async fn on_notification(&self, mut input: ServerModuleNotificationEventInput) -> ServerModuleNotificationEventOutput {
        println!("{:?}", input.source());

        let mut stream = Box::pin(input.body_mut().stream());

        while let Some(bytes) = stream.next().await {
            println!("{:?}", bytes);
        }
    }
}

impl TestServerModule {
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

            ServerTask::new(
                listener_address,
                match command {
                    ServerCommand::Leader => None,
                    ServerCommand::Follower {
                        leader_address,
                        server_id,
                    } => Some((leader_address, server_id)),
                },
                vec![Box::<dyn ServerModule>::from(Box::new(
                    TestServerModule::new(),
                ))]
                .into_iter(),
            )
            .await?
            .run()
            .await
        }
        MainCommand::Client { address, command } => {
            let client_task = ClientTask::new(
                address,
                vec![Box::<dyn ClientModule>::from(Box::new(
                    TestClientModule::new(),
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
