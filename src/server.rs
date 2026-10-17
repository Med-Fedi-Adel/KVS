use std::{
    io::BufReader,
    net::{SocketAddr, TcpListener, TcpStream},
};

use crate::{
    Result,
    engines::KvsEngine,
    protocol::{Request, Response, read_message, write_message},
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
    let request_result = {
        let read_stream = stream.try_clone()?;
        let mut reader = BufReader::new(read_stream);
        read_message::<_, Request>(&mut reader)
    };

    let response = match request_result {
        Ok(request) => match execute_request(request, engine) {
            Ok(response) => response,
            Err(error) => Response::Error(error.to_string()),
        },
        Err(error) => Response::Error(error.to_string()),
    };

    write_message(&mut stream, &response)?;
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
