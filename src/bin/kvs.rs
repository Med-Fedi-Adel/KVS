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

/*fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            println!("{error}");
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
*/

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

use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
};

fn write_pong<W: Write>(w: &mut W) -> io::Result<()> {
    w.write_all(b"+PONG\r\n")?;
    w.flush()
}

fn handle_connection(stream: TcpStream) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = BufWriter::new(stream);

    loop {
        let mut first = String::new();
        if reader.read_line(&mut first)? == 0 {
            break;
        }

        if !first.starts_with('*') {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "expected array"));
        }

        let mut bulk = String::new();
        reader.read_line(&mut bulk)?;

        let mut cmd = String::new();
        reader.read_line(&mut cmd);

        let cmd = cmd.trim_end_matches("\r\n");
        if cmd.eq_ignore_ascii_case("PING") {
            println!("PING Received");
            write_pong(&mut writer)?;
        } else {
            writer.write_all(b"-ERR only PING supported\r\n")?;
            writer.flush()?;
        }
    }

    Ok(())
}

fn run_server(addr: &str) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("listenning on {addr}");

    for conn in listener.incoming() {
        let stream = conn?;
        std::thread::spawn(move || {
            let _ = handle_connection(stream);
        });
    }

    Ok(())
}

fn write_ping<W: Write>(w: &mut W) -> io::Result<()> {
    w.write_all(b"*1\r\n$4\r\nPING\r\n")?;
    w.flush()
}

fn read_simple_string<R: BufRead>(r: &mut R) -> io::Result<String> {
    let mut line = String::new();
    r.read_line(&mut line)?;
    if !line.starts_with('+') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "expected simple string",
        ));
    }

    Ok(line.trim_end_matches("\r\n")[1..].to_string())
}

fn run_client(addr: &str) -> io::Result<()> {
    let stream = TcpStream::connect(addr)?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = BufWriter::new(stream);

    write_ping(&mut writer)?;

    let reply = read_simple_string(&mut reader)?;
    println!("reply: {reply}");
    Ok(())
}

fn main() -> io::Result<()> {
    match std::env::args().nth(1).as_deref() {
        Some("server") => run_server("127.0.0.1:6379"),
        Some("client") => run_client("127.0.0.1:6379"),
        _ => {
            eprintln!("usage: main [server|client]");
            Ok(())
        }
    }
}
