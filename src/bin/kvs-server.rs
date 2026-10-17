use std::{
    fs::{self, OpenOptions},
    io::Write,
    net::SocketAddr,
    path::Path,
    process::ExitCode,
};

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
    let data_dir = Path::new(".");

    log::info!("kvs-server version {}", env!("CARGO_PKG_VERSION"));
    log::info!(
        "configuration: addr={}, engine={}",
        cli.addr,
        engine_name.as_str()
    );

    check_engine(data_dir, engine_name.as_str())?;

    let engine: Box<dyn KvsEngine> = match engine_name {
        EngineName::Kvs => Box::new(engines::KvStore::open(data_dir)?),
        EngineName::Sled => {
            let sled_path = data_dir.join("sled_db");
            Box::new(engines::SledKvsEngine::open(&sled_path)?)
        }
    };

    record_engine(data_dir, engine_name.as_str())?;

    server::run(cli.addr, engine)
}

fn check_engine(data_dir: &Path, requested: &str) -> Result<()> {
    let marker_path = data_dir.join(".kvs-engine");

    match fs::read_to_string(&marker_path) {
        Ok(stored) => {
            let stored = stored.trim();
            if stored != requested {
                return Err(KvsError::Server(format!(
                    "database was opened with engine '{stored}', not '{requested}'"
                )));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // Support existing databases created before the marker was added.
            if let Some(stored) = detect_existing_engine(data_dir)?
                && stored != requested
            {
                return Err(KvsError::Server(format!(
                    "existing database uses engine '{stored}', not '{requested}'"
                )));
            }
        }
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

fn detect_existing_engine(data_dir: &Path) -> Result<Option<&'static str>> {
    let kvs_log = data_dir.join("kvs.log");
    let kvs_has_data = match fs::metadata(&kvs_log) {
        Ok(metadata) => metadata.len() > 0,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };

    let sled_dir = data_dir.join("sled_db");
    let sled_has_data = match fs::read_dir(&sled_dir) {
        Ok(mut entries) => entries.next().transpose()?.is_some(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };

    match (kvs_has_data, sled_has_data) {
        (true, false) => Ok(Some("kvs")),
        (false, true) => Ok(Some("sled")),
        (true, true) => Err(KvsError::Server(
            "found data for both storage engines, but no engine marker".to_owned(),
        )),
        (false, false) => Ok(None),
    }
}

fn record_engine(data_dir: &Path, requested: &str) -> Result<()> {
    let marker_path = data_dir.join(".kvs-engine");

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&marker_path)
    {
        Ok(mut file) => {
            writeln!(file, "{requested}")?;
            file.sync_all()?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            // If another server created the marker between check and write,
            // verify that it agrees with this startup.
            let stored = fs::read_to_string(&marker_path)?;
            if stored.trim() == requested {
                Ok(())
            } else {
                Err(KvsError::Server(format!(
                    "database was opened with engine '{}', not '{requested}'",
                    stored.trim()
                )))
            }
        }
        Err(error) => Err(error.into()),
    }
}
