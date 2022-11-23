#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Parser, Subcommand};
use std::error::Error;
use std::iter::once;
use std::path::PathBuf;
use zlambda_client::Client;
use zlambda_common::library::Library;
use zlambda_common::runtime::Runtime;
use zlambda_server::Server;

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct MainArguments {
    #[clap(subcommand)]
    command: MainCommand,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum MainCommand {
    Server {
        #[clap(default_value = "0.0.0.0:8000")]
        listener_address: String,
        leader_address: Option<String>,
    },
    Client {
        #[clap(default_value = "0.0.0.0:8000")]
        address: String,
        #[clap(subcommand)]
        command: ClientCommand,
    },
    Module {
        path: PathBuf,
        commands: Vec<String>,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum ClientCommand {
    Load,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = MainArguments::try_parse()?;

    match arguments.command {
        MainCommand::Server {
            listener_address,
            leader_address,
        } => {
            tracing_subscriber::fmt::init();

            let runtime = Runtime::new()?;

            runtime.block_on(async move {
                let server = match Server::new(listener_address, leader_address).await {
                    Err(error) => return Err(error),
                    Ok(server) => server,
                };

                server.run().await;

                Ok(())
            })?;
        }
        MainCommand::Client { address, command } => {
            let runtime = Runtime::new()?;

            runtime.block_on(async move {
                let client = match Client::new(address).await {
                    Err(error) => return Err(error),
                    Ok(client) => client,
                };

                match command {
                    Load => {}
                };

                Ok(())
            })?;
        }
        MainCommand::Module { path, commands } => {
            let library = Library::load(&path)?;
            let runtime = Runtime::new()?;

            let arguments = once(path.display().to_string())
                .chain(commands.into_iter())
                .collect::<Vec<_>>();

            runtime.block_on(async move {
                library.module().on_command(arguments).await;
            });
        }
    };

    Ok(())
}
