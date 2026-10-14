use std::net::SocketAddr;

use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "kvs-server", version)]
struct Args {
    #[arg(long, default_value = "127.0.01:4000")]
    addr: SocketAddr,

    engine: Option<EngineName>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum EngineName {
    Kvs,
    Sled,
}

fn main() {
    let args = Args::parse();

    println!("Listening address: {}", args.addr);
    if let Some(engine) = args.engine {
        println!("Requested engine: {engine:?}");
    }
}
