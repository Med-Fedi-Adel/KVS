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

impl EngineName {
    fn as_str(self) -> &'static str {
        match self {
            Self::Kvs => "kvs",
            Self::Sled => "sled",
        }
    }
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stderr)
        .init();

    let cli = Cli::parse();
    let engine_name = cli.engine.unwrap_or(EngineName::Kvs);

    log::info!("kvs-server version {}", env!("CARGO_PKG_VERSION"));
    log::info!(
        "configuration: addr={}, engine={}",
        cli.addr,
        engine_name.as_str()
    );

    let engine: Box<dyn KvsEngine> = match engine_name {
        EngineName::Kvs => Box::new(engines::KvStore::open(Path::new("."))?),
        EngineName::Sled => {
            return Err(KvsError::Server(
                "the sled engine is not implemented yet".to_owned(),
            ));
        }
    };

    server::run(cli.addr, engine)
}
