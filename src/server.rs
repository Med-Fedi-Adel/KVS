use std::{
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpListener, TcpStream},
};

use crate::{
    Result,
    engines::KvsEngine,
    protocol::{Request, Response},
};

pub fn run(addr: SocketAddr, mut engine: Box<dyn KvsEngine>) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    log::info!("listening on {addr}");

    for incoming in listener.incoming() {
        let stream = match incoming {
            Ok(stream) => stream,
            Err(error) => {
                log::error!("failed to accept connection: {error}");
                continue;
            }
        };

        match stream.peer_addr() {
            Ok(peer) => log::info!("accepted connection from {peer}"),
            Err(error) => log::warn!("accepted connection, but couldn't get peer address: {error}"),
        }

        handle_connection(stream, engine.as_mut())?;
    }

    Ok(())
}

pub fn handle_connection(mut stream: TcpStream, engine: &mut dyn KvsEngine) -> Result<()> {
    let mut line = Vec::new();
    BufReader::new(&mut stream).read_until(b'\n', &mut line)?;

    let response = match serde_json::from_slice::<Request>(&line) {
        Ok(request) => match execute_request(request, engine) {
            Ok(response) => response,
            Err(error) => Response::Error(error.to_string()),
        },
        Err(error) => Response::Error(error.to_string()),
    };

    serde_json::to_writer(&mut stream, &response)?;
    stream.write_all(b"\n")?;

    Ok(())
}

fn execute_request(request: Request, engine: &mut dyn KvsEngine) -> Result<Response> {
    match request {
        Request::Set { key, value } => {
            engine.set(key, value)?;
            Ok(Response::Ok)
        }
        Request::Get { key } => Ok(Response::Value(engine.get(key)?)),
        Request::Rm { key } => {
            engine.remove(key)?;
            Ok(Response::Ok)
        }
    }
}
