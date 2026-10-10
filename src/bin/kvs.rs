use clap::{Parser, Subcommand};
use kvs::{KvStore, Result};
use serde::Serialize;
use std::{
    path::Path,
    process::{self, ExitCode},
};

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
    let mut store = KvStore::open(Path::new("."))?;

    match cli.command {
        Command::Set { key, value } => store.set(key, value)?,
        Command::Get { key } => match store.get(key)? {
            Some(value) => println!("{value}"),
            None => println!("Key not found"),
        },
        Command::Rm { key } => store.remove(key)?,
    }

    Ok(())
}
