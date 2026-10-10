use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Result, error::KvsError};

/// A key-value store backed by an append-only log.
///
/// The store keeps an in-memory index from keys to the byte offsets of their
/// latest `Set` records. Values are read from the log when requested.
pub struct KvStore {
    index: HashMap<String, u64>,
    log_path: PathBuf,
}

impl KvStore {
    /// Creates a store that uses `kvs.log` in the current working directory.
    pub fn new() -> KvStore {
        KvStore {
            index: HashMap::new(),
            log_path: PathBuf::from("kvs.log"),
        }
    }

    /// Opens the store in `path`, creating `kvs.log` if it does not exist.
    ///
    /// Rebuilds the in-memory key-to-offset index by replaying the log.
    pub fn open(path: &Path) -> Result<KvStore> {
        let log_path = path.join("kvs.log");

        // Create the log if it doesn't exist.
        drop(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)?,
        );

        let file = File::open(&log_path)?;
        let mut reader = BufReader::new(file);
        let mut index = HashMap::new();
        let mut offset = 0_u64;
        let mut line = Vec::new();

        loop {
            line.clear();
            let bytes_read = reader.read_until(b'\n', &mut line)?;

            if bytes_read == 0 {
                break;
            }

            let record: LogRecord = serde_json::from_slice(&line)?;

            match record {
                LogRecord::Set { key, .. } => {
                    index.insert(key, offset);
                }
                LogRecord::Rm { key } => {
                    index.remove(&key);
                }
            }

            offset += bytes_read as u64;
        }

        Ok(KvStore { index, log_path })
    }

    /// Sets the value for `key`, replacing any existing value.
    ///
    /// Appends the command to the log and updates the in-memory index.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let record = LogRecord::Set {
            key: key.clone(),
            value,
        };

        let offset = self.append_log(&record)?;
        self.index.insert(key, offset);

        Ok(())
    }

    /// Returns the value for `key`, or `None` if the key is not present.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        let Some(&offset) = self.index.get(&key) else {
            return Ok(None);
        };

        let file = File::open(&self.log_path)?;
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(offset))?;

        let mut line = Vec::new();
        reader.read_until(b'\n', &mut line)?;

        let record: LogRecord = serde_json::from_slice(&line)?;

        match record {
            LogRecord::Set { value, .. } => Ok(Some(value)),
            LogRecord::Rm { .. } => Ok(None),
        }
    }

    /// Removes `key` from the store.
    ///
    /// Returns [`KvsError::KeyNotFound`] if the key does not exist.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.index.contains_key(&key) {
            return Err(KvsError::KeyNotFound);
        }

        self.append_log(&LogRecord::Rm { key: key.clone() })?;
        self.index.remove(&key);

        Ok(())
    }

    fn append_log(&self, record: &LogRecord) -> Result<u64> {
        let mut line = serde_json::to_vec(record)?;
        line.push(b'\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let offset = file.metadata()?.len();
        file.write_all(&line)?;

        Ok(offset)
    }
}

#[derive(Serialize, Deserialize)]
enum LogRecord {
    Set { key: String, value: String },
    Rm { key: String },
}
