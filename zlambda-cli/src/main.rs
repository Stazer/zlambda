#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Parser, Command, Subcommand, FromArgMatches, ArgMatches};
use std::error::Error;
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
        #[clap(subcommand)]
        command: ModuleCommand,
    },
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum ClientCommand {
    Load,
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
struct ModuleCommand {

}

impl FromArgMatches for ModuleCommand {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        Self {}
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        Ok(())
    }
}

impl Subcommand for ModuleCommand {
    fn augment_subcommands(command: Command) -> Command {
        command
    }

    fn augment_subcommands_for_update(command: Command) -> Command {
        command
    }

    fn has_subcommand(name: &str) -> bool {
        false
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn main() -> Result<(), Box<dyn Error>> {
    let arguments = MainArguments::parse();

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
                    Load => {
                    }
                };

                Ok(())

            })?;
        }
        MainCommand::Module { .. } => {
            /*let library = Library::load(&path)?;
            let modules = library.modules()?;*/
        }
    };

    Ok(())
}
