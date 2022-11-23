use clap::{Parser, Subcommand};
use std::error::Error;
use zlambda_common::async_trait::async_trait;
use serde::{Deserialize, Serialize};
use zlambda_common::module::{DispatchEvent, Module};

////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize, Serialize)]
struct DispatchPayload {
    program: String,
    arguments: Vec<String>,
}

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
    Dispatch {
        program: String,
        arguments: Vec<String>,
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ModuleImplementation {}

#[async_trait]
impl Module for ModuleImplementation {
    async fn on_command(&self, arguments: Vec<String>) -> Result<(), Box<dyn Error>> {
        let arguments = MainArguments::try_parse_from(arguments)?;

        match arguments.command {
            MainCommand::Dispatch { program, arguments } => {

            }
        }

        Ok(())
    }

    async fn on_dispatch(&self, event: DispatchEvent) {

    }
}

#[no_mangle]
pub extern "C" fn module() -> Box<dyn Module> {
    Box::new(ModuleImplementation {})
}
