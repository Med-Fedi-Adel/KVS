#![deny(missing_docs)]
//! A simple key/value store.
pub use kv::KvStore;

pub use result::Result;

mod kv;
mod result;
