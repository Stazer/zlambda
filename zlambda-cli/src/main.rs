#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Args, Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;
//use tokio::io::AsyncWriteExt;
//use tokio::io::{stdin, stdout};
use zlambda_core::common::module::ModuleId;
use zlambda_core::server::{ServerId, ServerTask};
use std::iter::empty;

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
    Load { path: PathBuf },
    Dispatch { id: ModuleId },
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
                empty(),
            )
            .await?
            .run()
            .await
        }
        MainCommand::Client {
            address: _,
            command: _,
        } => {
            /*let mut client = match Client::new(address).await {
                Err(error) => return Err(error),
                Ok(client) => client,
            };

            match command {
                ClientCommand::Load { path } => {
                    let id = client.load_module(&path).await?;

                    println!("{}", id);
                }
                ClientCommand::Dispatch { id } => {
                    stdout()
                        .write_all(&client.dispatch(id, stdin()).await?)
                        .await?;
                }
            };*/
        }
    };

    Ok(())
}
