#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Parser, Subcommand};
use std::error::Error;
use std::iter::once;
use std::path::PathBuf;
use zlambda_client::Client;
use zlambda_common::module::{ReadModuleEventInput, Module, ModuleEventDispatchPayload, ModuleId};
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
}

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Subcommand)]
enum ClientCommand {
    Load {
        path: PathBuf,
    },
    Dispatch {
        id: ModuleId,
        path: PathBuf,
        commands: Vec<String>,
    },
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
                let mut client = match Client::new(address).await {
                    Err(error) => return Err(error),
                    Ok(client) => client,
                };

                match command {
                    ClientCommand::Load { path } => {
                        let id = client.load_module(&path).await?;

                        println!("{}", id);
                    }
                    ClientCommand::Dispatch { id, path, commands } => {
                        let arguments = once(path.display().to_string())
                            .chain(commands.into_iter())
                            .collect::<Vec<_>>();

                        let module = Module::load(0, &path)?;

                        let (payload, ) = module.event_listener().read(
                            ReadModuleEventInput::new(arguments)
                        ).await?.into();

                        let payload = client.dispatch(id, payload).await?;
                    }
                };

                Ok(())
            })?;
        }
    };

    Ok(())
}
