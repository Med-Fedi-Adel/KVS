use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Result, error::KvsError};

/// The `KvStore` stores string key/value pairs in an append-only log.
///
/// It keeps an in-memory index mapping keys to the offsets of their latest
/// `Set` records in the log.
/// Example:
///
/// ```rust
/// # fn main() -> kvs::Result<()> {
/// # use kvs::KvStore;
/// let mut store = KvStore::new();
/// store.set("key".to\_owned(), "value".to\_owned())?;
/// let val = store.get("key".to\_owned())?;
/// assert\_eq!(val, Some("value".to\_owned()));
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct KvStore {
    index: HashMap<String, u64>,
    log_path: PathBuf,
}

impl KvStore {
    /// Creates a `KvStore`.
    pub fn new() -> KvStore {
        KvStore {
            index: HashMap::new(),
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
            index: HashMap::new(),
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

        let offset = self.append_log(&log)?;
        self.index.insert(key, offset);

        Ok(())
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        let Some(&offset) = self.index.get(&key) else {
            return Ok(None);
        };

        let file = File::open(&self.log_path)?;
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(offset))?;

        let mut line = String::new();
        reader.read_line(&mut line)?;

        let record: LogRecord = serde_json::from_str(&line)?;

        match record {
            LogRecord::Set { value, .. } => Ok(Some(value)),
            LogRecord::Rm { .. } => Ok(None),
        }
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.index.contains_key(&key) {
            return Err(KvsError::KeyNotFound);
        }

        let log = LogRecord::Rm { key: key.clone() };
        self.append_log(&log)?;
        self.index.remove(&key);

        Ok(())
    }

    fn append_log(&self, log: &LogRecord) -> Result<u64> {
        let mut line = serde_json::to_string(log)?;
        line.push('\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let offset = file.metadata()?.len();
        file.write_all(line.as_bytes())?;
        Ok(offset)
    }
}

// JSON Lines keeps each log command human-readable and independently parseable.
#[derive(Serialize, Deserialize)]
enum LogRecord {
    Set { key: String, value: String },
    Rm { key: String },
}
