use std::io;

#[derive(Debug)]
pub enum KvsError {
    Io(io::Error),
}

impl From<io::Error> for KvsError {
    fn from(error: io::Error) -> Self {
        KvsError::Io(error)
    }
}

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, KvsError>;
