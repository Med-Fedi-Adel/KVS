use crate::Result;

/// A storage engine for the key-value store.
pub trait KvsEngine {
    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    fn set(&mut self, key: String, value: String) -> Result<()>;

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    fn get(&mut self, key: String) -> Result<Option<String>>;

    /// Removes a given key.
    ///
    /// # Errors
    ///
    /// Returns `KvsError::KeyNotFound` if the key is not found.
    fn remove(&mut self, key: String) -> Result<()>;
}

pub mod kvs;
pub use kvs::KvStore;

pub mod sled;
pub use sled::SledKvsEngine;
// Add this after src/engines/sled.rs exists:
// pub mod sled;
