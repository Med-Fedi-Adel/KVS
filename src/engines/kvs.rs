use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{Result, engines::KvsEngine, error::KvsError};

const COMPACTION_MIN_LOG_BYTES: u64 = 1024 * 1024;

impl KvsEngine for KvStore {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        KvStore::set(self, key, value)
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        KvStore::get(self, key)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        KvStore::remove(self, key)
    }
}

#[derive(Clone, Copy)]
struct LogPointer {
    offset: u64,
    len: u64,
}

/// A key-value store backed by an append-only log.
///
/// The store keeps an in-memory index from keys to the byte offsets of their
/// latest `Set` records. Values are read from the log when requested.
pub struct KvStore {
    index: HashMap<String, LogPointer>,
    log_path: PathBuf,
    log_size: u64,
    live_bytes: u64,
}

impl KvStore {
    /// Creates a store that uses `kvs.log` in the current working directory.
    pub fn new() -> KvStore {
        KvStore {
            index: HashMap::new(),
            log_path: PathBuf::from("kvs.log"),
            log_size: 0u64,
            live_bytes: 0u64,
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
        let mut offset = 0u64;
        let mut live_bytes = 0u64;
        let mut line = Vec::new();

        loop {
            line.clear();
            let bytes_read = reader.read_until(b'\n', &mut line)?;

            if bytes_read == 0 {
                break;
            }

            let record: LogRecord = serde_json::from_slice(&line)?;
            let len = bytes_read as u64;

            match record {
                LogRecord::Set { key, .. } => {
                    if let Some(previous) = index.insert(key, LogPointer { offset, len }) {
                        live_bytes -= previous.len;
                    }
                    live_bytes += len;
                }
                LogRecord::Rm { key } => {
                    if let Some(previous) = index.remove(&key) {
                        live_bytes -= previous.len;
                    }
                }
            }

            offset += len;
        }

        Ok(KvStore {
            index,
            log_path,
            log_size: offset,
            live_bytes,
        })
    }

    /// Sets the value for `key`, replacing any existing value.
    ///
    /// Appends the command to the log and updates the in-memory index.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let record = LogRecord::Set {
            key: key.clone(),
            value,
        };

        let pointer = self.append_log(&record)?;

        if let Some(previous) = self.index.insert(key, pointer) {
            self.live_bytes -= previous.len;
        }

        self.live_bytes += pointer.len;
        self.log_size = pointer.offset + pointer.len;

        self.maybe_compact()
    }

    /// Returns the value for `key`, or `None` if the key is not present.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        let Some(pointer) = self.index.get(&key) else {
            return Ok(None);
        };

        // this is also why the benchmarking is total shit for kvs repeated open and metadata calls
        // create this overhead on the read benchmarks as well
        // we need an in memory index probably for values
        let file = File::open(&self.log_path)?;
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(pointer.offset))?;

        let mut line = Vec::with_capacity(pointer.len as usize);
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
        let Some(previous) = self.index.get(&key).copied() else {
            return Err(KvsError::KeyNotFound);
        };

        let pointer = self.append_log(&LogRecord::Rm { key: key.clone() })?;
        self.index.remove(&key);
        self.live_bytes -= previous.len;
        self.log_size = pointer.offset + pointer.len;

        self.maybe_compact()
    }

    fn append_log(&self, record: &LogRecord) -> Result<LogPointer> {
        let mut line = serde_json::to_vec(record)?;
        line.push(b'\n');

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        let offset = file.metadata()?.len();
        file.write_all(&line)?;
        // this why it's benchmarking slower than sled
        file.sync_data()?;

        Ok(LogPointer {
            offset,
            len: line.len() as u64,
        })
    }

    fn maybe_compact(&mut self) -> Result<()> {
        let obsolete_bytes = self.log_size.saturating_sub(self.live_bytes);

        if self.log_size >= COMPACTION_MIN_LOG_BYTES && obsolete_bytes >= self.live_bytes {
            self.compact()?;
        }

        Ok(())
    }

    fn compact(&mut self) -> Result<()> {
        // Keep the temporary file beside the log so the rename stays on the same filesystem
        let temp_path = self.log_path.with_extension("compact.tmp");

        let mut old_log = BufReader::new(File::open(&self.log_path)?);
        let mut new_log = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp_path)?;

        // Fdiw noticed hashmap is unordered. Sort by offset to read the old offset first
        let mut entries: Vec<(String, LogPointer)> = self
            .index
            .iter()
            .map(|(key, pointer)| (key.clone(), *pointer))
            .collect();
        entries.sort_by_key(|(_, pointer)| pointer.offset);

        let mut new_index = HashMap::with_capacity(entries.len());
        let mut new_offset = 0u64;

        for (key, pointer) in entries {
            old_log.seek(SeekFrom::Start(pointer.offset))?;

            let mut record = vec![0; pointer.len as usize];
            old_log.read_exact(&mut record)?;
            new_log.write_all(&record)?;

            new_index.insert(
                key,
                LogPointer {
                    offset: new_offset,
                    len: pointer.len,
                },
            );
            new_offset += pointer.len;
        }

        new_log.flush()?;
        new_log.sync_all()?;
        drop(new_log);

        std::fs::rename(&temp_path, &self.log_path)?;

        self.index = new_index;
        self.log_size = new_offset;
        self.live_bytes = new_offset;

        Ok(())
    }
}

impl Default for KvStore {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Serialize, Deserialize)]
enum LogRecord {
    Set { key: String, value: String },
    Rm { key: String },
}
