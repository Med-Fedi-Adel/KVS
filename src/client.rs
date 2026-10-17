use std::{
    io::{self, BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
};

use crate::{
    KvsError, Result,
    protocol::{Request, Response, read_message, write_message},
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
    write_message(&mut stream, &request)?;

    let mut reader = BufReader::new(stream);
    let response: Response = read_message(&mut reader)?;

    match response {
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
