#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Parser, Subcommand};
use std::error::Error;
use std::path::PathBuf;
use tracing_subscriber::fmt::init;
use zlambda_common::Library;
use zlambda_server::cluster::node::ClusterNode;
use zlambda_server::runtime::Runtime;

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
    Client,
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
            init();

            let runtime = Runtime::new()?;

            runtime.block_on(async move {
                let mut node = match ClusterNode::new(listener_address, leader_address).await {
                    Ok(sender) => sender,
                    Err(error) => return Err(error),
                };

                node.main().await;

                Ok(())
            })?;

            let _ = runtime.enter();
        }
        MainCommand::Client => {}
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
