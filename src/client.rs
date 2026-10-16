use std::{
    io::{self, BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
};

use crate::{
    KvsError, Result,
    protocol::{Request, Response},
};

pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

pub fn execute(addr: SocketAddr, command: Command) -> Result<()> {
    let request = match command {
        Command::Set { key, value } => Request::Set { key, value },
        Command::Get { key } => Request::Get { key },
        Command::Rm { key } => Request::Rm { key },
    };

    let mut stream = TcpStream::connect(addr)?;
    serde_json::to_writer(&mut stream, &request)?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut response_line = String::new();
    BufReader::new(stream).read_line(&mut response_line)?;

    if response_line.is_empty() {
        return Err(KvsError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "server closed the connection without responding",
        )));
    }

    match serde_json::from_str::<Response>(&response_line)? {
        Response::Ok => Ok(()),
        Response::Value(Some(value)) => {
            println!("{value}");
            Ok(())
        }
        Response::Value(None) => {
            println!("Key not found");
            Ok(())
        }
        Response::Error(message) => Err(KvsError::Server(message)),
    }
}
