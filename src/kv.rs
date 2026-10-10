use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::Result;

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are stored in a `HashMap` in memory and not persisted to disk.
///
/// Example:
///
/// ```rust
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key".to_owned(), "value".to_owned());
/// let val = store.get("key".to_owned());
/// assert_eq!(val, Some("value".to_owned()));
/// ```
#[derive(Default)]
pub struct KvStore {
    map: HashMap<String, String>,
    log_path: PathBuf,
}

impl KvStore {
    /// Creates a `KvStore`.
    pub fn new() -> KvStore {
        KvStore {
            map: HashMap::new(),
            log_path: PathBuf::from("kvs.log"),
        }
    }

    #[allow(missing_docs)]
    pub fn open(path: &Path) -> Result<KvStore> {
        let log_path = path.join("kvs.log");

        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        Ok(KvStore {
            map: HashMap::new(),
            log_path,
        })
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let log = LogRecord::Set {
            key: key.clone(),
            value: value.clone(),
        };

        self.append_log(&log)?;
        self.map.insert(key, value);

        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.map.get(&key).cloned())
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.map.remove(&key);

        Ok(())
    }

    fn append_log(&self, log: &LogRecord) -> Result<()> {
        let mut line = serde_json::to_string(log)?;
        line.push('\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        file.write_all(line.as_bytes())?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
enum LogRecord {
    Set { key: String, value: String },
    Rm { key: String },
}
