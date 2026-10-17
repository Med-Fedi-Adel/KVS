use std::path::Path;

use sled::Db;

use crate::{KvsEngine, KvsError, Result};

pub struct SledKvsEngine {
    db: Db,
}

impl SledKvsEngine {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            db: sled::open(path)?,
        })
    }
}

impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        self.db.insert(key.as_bytes(), value.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    fn get(&mut self, key: String) -> Result<Option<String>> {
        match self.db.get(key.as_bytes())? {
            Some(value) => Ok(Some(String::from_utf8(value.to_vec())?)),
            None => Ok(None),
        }
    }

    fn remove(&mut self, key: String) -> Result<()> {
        match self.db.remove(key.as_bytes())? {
            Some(_) => {
                self.db.flush()?;
                Ok(())
            }
            None => Err(KvsError::KeyNotFound),
        }
    }
}
