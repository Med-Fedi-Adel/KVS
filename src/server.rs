use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::Path,
};

use crate::{
    KvStore, Result,
    protocol::{Request, Response},
};

pub fn run(addr: SocketAddr, path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;

    let listener = TcpListener::bind(addr)?;
    let mut store = KvStore::open(path)?;

    for stream in listener.incoming() {
        handle_connection(stream?, &mut store)?;
    }

    Ok(())
}

pub fn handle_connection(mut stream: TcpStream, store: &mut KvStore) -> Result<()> {
    let mut line = Vec::new();
    BufReader::new(&mut stream).read_until(b'\n', &mut line)?;

    let response = match serde_json::from_slice::<Request>(&line) {
        Ok(request) => match execute_request(request, store) {
            Ok(response) => response,
            Err(error) => Response::Error(error.to_string()),
        },
        Err(error) => Response::Error(error.to_string()),
    };

    serde_json::to_writer(&mut stream, &response)?;
    stream.write_all(b"\n")?;

    Ok(())
}

fn execute_request(request: Request, store: &mut KvStore) -> Result<Response> {
    match request {
        Request::Set { key, value } => {
            store.set(key, value)?;
            Ok(Response::Ok)
        }
        Request::Get { key } => Ok(Response::Value(store.get(key)?)),
        Request::Rm { key } => {
            store.remove(key)?;
            Ok(Response::Ok)
        }
    }
}
