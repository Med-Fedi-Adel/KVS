use clap::{Parser, Subcommand};
use kvs::{Result, client};
use serde::Serialize;
use std::{net::SocketAddr, process::ExitCode};

#[derive(Parser)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    disable_help_subcommand = true,
    subcommand_required = true,
    arg_required_else_help = true
)]
struct Cli {
    #[arg(long, global = true, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Serialize)]
pub enum Command {
    /// Set the value of a string key to a string
    Set {
        /// A string key
        key: String,
        /// The string value of the key
        value: String,
    },

    /// Get the string value of a given string key
    Get {
        /// A string key
        key: String,
    },

    /// Remove a given key
    Rm {
        /// A string key
        key: String,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let command = match cli.command {
        Command::Set { key, value } => client::Command::Set { key, value },
        Command::Get { key } => client::Command::Get { key },
        Command::Rm { key } => client::Command::Rm { key },
    };

    client::execute(cli.addr, command)
}
