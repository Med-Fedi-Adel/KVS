use std::io;

#[derive(Debug)]
pub enum KvsError {
    Io(io::Error),
    Serde(serde_json::Error),
}

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
