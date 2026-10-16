use std::{net::SocketAddr, path::Path, process::ExitCode};

use clap::{Parser, ValueEnum};
use kvs::{
    KvsError, Result,
    engines::{self, KvsEngine},
    server,
};

#[derive(Parser)]
#[command(
    name = "kvs-server",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    disable_help_subcommand = true
)]
struct Cli {
    /// IP address and port to listen on
    #[arg(long, default_value = "127.0.0.1:4000")]
    addr: SocketAddr,

    /// Storage engine to use
    #[arg(long, value_enum)]
    engine: Option<EngineName>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum EngineName {
    Kvs,
    Sled,
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
    let data_dir = Path::new(".");

    let engine: Box<dyn KvsEngine> = match cli.engine {
        Some(EngineName::Sled) => {
            return Err(KvsError::Server(
                "the sled engine is not implemented yet".to_owned(),
            ));
        }
        Some(EngineName::Kvs) | None => Box::new(engines::KvStore::open(data_dir)?),
    };

    server::run(cli.addr, engine)
}
