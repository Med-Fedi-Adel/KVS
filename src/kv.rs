use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Result, error::KvsError};

/// A key-value store backed by an append-only log.
///
/// When opened, the store replays the log into an in-memory map. Changes made
/// with [`set`](KvStore::set) and [`remove`](KvStore::remove) are appended to
/// the log, and [`get`](KvStore::get) reads from the map.
///
/// # Example
///
/// ```rust
/// # fn main() -> kvs::Result<()> {
/// # use kvs::KvStore;
/// let dir = tempfile::tempdir().expect("create temporary directory");
/// let mut store = KvStore::open(dir.path())?;
///
/// store.set("key".to\_owned(), "value".to\_owned())?;
/// let value = store.get("key".to\_owned())?;
/// assert\_eq!(value, Some("value".to\_owned()));
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct KvStore {
    map: HashMap<String, String>,
    log_path: PathBuf,
}

impl KvStore {
    /// Creates an empty in-memory store that appends its log to `kvs.log`
    /// in the current working directory.
    pub fn new() -> KvStore {
        KvStore {
            map: HashMap::new(),
            log_path: PathBuf::from("kvs.log"),
        }
    }

    /// Opens the store in `path`, creating its log file if necessary.
    ///
    /// Replays the log into the store's in-memory map.
    pub fn open(path: &Path) -> Result<KvStore> {
        let log_path = path.join("kvs.log");

        // Ensure the log exists, then close this handle before reading it.
        drop(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)?,
        );

        let file = File::open(&log_path)?;
        let reader = BufReader::new(file);
        let records = serde_json::Deserializer::from_reader(reader).into_iter::<LogRecord>();

        let mut store = KvStore {
            map: HashMap::new(),
            log_path,
        };

        // Replay each record into the map, one at a time.
        for record in records {
            match record? {
                LogRecord::Set { key, value } => {
                    store.map.insert(key, value);
                }
                LogRecord::Rm { key } => {
                    store.map.remove(&key);
                }
            }
        }

        Ok(store)
    }

    /// Sets the value for `key`, replacing any existing value.
    ///
    /// The command is appended to the log before the in-memory map is updated.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let record = LogRecord::Set {
            key: key.clone(),
            value: value.clone(),
        };

        self.append_log(&record)?;
        self.map.insert(key, value);
        Ok(())
    }

    /// Returns the value for `key`, or `None` if the key is not present.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.map.get(&key).cloned())
    }

    /// Removes `key` from the store.
    ///
    /// Returns [`KvsError::KeyNotFound`] if the key does not exist.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.map.contains_key(&key) {
            return Err(KvsError::KeyNotFound);
        }

        self.append_log(&LogRecord::Rm { key: key.clone() })?;
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
