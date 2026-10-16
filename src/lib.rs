#![deny(missing_docs)]
//! A simple key/value store.
pub use kv::KvStore;

pub use error::KvsError;
pub use error::Result;

#[allow(missing_docs)]
pub mod client;
mod error;
mod kv;
mod protocol;
#[allow(missing_docs)]
pub mod server;
