#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;
use zlambda_common::library::Library;
use zlambda_server::node::Node;
use zlambda_common::runtime::Runtime;
use zlambda_client::Client;

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
        address: String,
    },
    Module {
        path: PathBuf,
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
                let node = match Node::new(listener_address, leader_address).await {
                    Err(error) => return Err(error),
                    Ok(node) => node,
                };

                node.run().await;

                Ok(())
            })?;
        }
        MainCommand::Client { address } => {
            let runtime = Runtime::new()?;

            runtime.block_on(async move {
                let client = Client::new(address).await;
            });
        }
        MainCommand::Module { path } => {
            let library = Library::load(&path)?;
            let modules = library.modules()?;
            let spec = modules
                .iter()
                .map(|x| x.specification())
                .collect::<Vec<_>>();
            println!("{:?}", spec);
        }
    };

    Ok(())
}
