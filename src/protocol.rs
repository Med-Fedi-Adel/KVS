use std::io::{BufRead, Write};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::KvsError;

#[derive(Serialize, Deserialize)]
pub enum Request {
    Set { key: String, value: String },
    Get { key: String },
    Rm { key: String },
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    Ok,
    Value(Option<String>),
    Error(String),
}

pub fn write_message<W, T>(writer: &mut W, message: &T) -> Result<(), KvsError>
where
    W: Write,
    T: Serialize,
{
    let bytes = serde_json::to_vec(message).map_err(KvsError::Serde)?;
    writer.write_all(&bytes).map_err(KvsError::Io)?;
    writer.write_all(b"\n").map_err(KvsError::Io)?;
    writer.flush().map_err(KvsError::Io)?;

    Ok(())
}

pub fn read_message<R, T>(reader: &mut R) -> Result<T, KvsError>
where
    R: BufRead,
    T: DeserializeOwned,
{
    let mut line = Vec::new();
    let bytes_read = reader.read_until(b'\n', &mut line).map_err(KvsError::Io)?;

    if bytes_read == 0 {
        return Err(KvsError::Server(
            "connection closed before a message was received".to_owned(),
        ));
    }

    if line.last() == Some(&b'\n') {
        line.pop();
    }

    serde_json::from_slice(&line).map_err(KvsError::Serde)
}
