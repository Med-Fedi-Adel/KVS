pub use engines::KvStore;
pub use engines::KvsEngine;
pub use error::KvsError;
pub use error::Result;

#[allow(missing_docs)]
pub mod client;
#[allow(missing_docs)]
pub mod engines;
mod error;
mod protocol;
#[allow(missing_docs)]
pub mod server;
