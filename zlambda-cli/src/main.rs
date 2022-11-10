#![feature(async_closure)]

////////////////////////////////////////////////////////////////////////////////////////////////////

use clap::{Parser, Subcommand};
use std::error::Error;
use tracing_subscriber::fmt::init;
use zlambda_server::cluster::NodeActor;
use zlambda_server::run::run_with_default_system_runner;
use std::path::PathBuf;
use zlambda_common::Library;

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

            run_with_default_system_runner(async move || {
                NodeActor::new(listener_address, leader_address).await?;
                Ok(())
            })
            .expect("Cannot start manager");
        }
        MainCommand::Client => {},
        MainCommand::Module { path } => {
            let library = Library::load(&path)?;
            let modules = library.modules()?;
            println!("{:?}", modules);
        }
    };

    Ok(())
}
