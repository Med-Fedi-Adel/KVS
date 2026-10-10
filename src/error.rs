use std::{error::Error, fmt, io};

#[derive(Debug)]
pub enum KvsError {
    Io(io::Error),
    Serde(serde_json::Error),
    KeyNotFound,
}

impl fmt::Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KvsError::Io(err) => write!(f, "{err}"),
            KvsError::Serde(err) => write!(f, "{err}"),
            KvsError::KeyNotFound => write!(f, "Key not found"),
        }
    }
}

impl Error for KvsError {}

impl From<io::Error> for KvsError {
    fn from(error: io::Error) -> Self {
        KvsError::Io(error)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(error: serde_json::Error) -> Self {
        KvsError::Serde(error)
    }
}

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, KvsError>;
