use clap::{Parser, Subcommand};
use kvs::{KvStore, KvsError, Result};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
    path::Path,
    process::ExitCode,
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

#[derive(Deserialize)]
enum ServerResponse {
    Ok,
    Value(Option<String>),
    Error(String),
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

    let mut stream = TcpStream::connect(cli.addr)?;
    serde_json::to_writer(&mut stream, &cli.command)?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut response_line = String::new();
    BufReader::new(stream).read_line(&mut response_line)?;

    if response_line.is_empty() {
        return Err(KvsError::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "server closed the connection without responding",
        )));
    }

    match serde_json::from_str::<ServerResponse>(&response_line)? {
        ServerResponse::Ok => Ok(()),
        ServerResponse::Value(Some(value)) => {
            println!("{value}");
            Ok(())
        }
        ServerResponse::Value(None) => {
            println!("Key not found");
            Ok(())
        }
        ServerResponse::Error(message) => Err(KvsError::Server(message)),
    }
}
