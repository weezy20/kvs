//! Sled wrapper for KvsEngine
use std::path::PathBuf;

use crate::{
    error::{DbError, Result},
    KvsEngine,
};
use log::warn;

/// Sled backend for KVS
pub struct SledKvsEngine {
    db: sled::Db,
}

impl SledKvsEngine {
    /// Start a Sled Kvs Engine
    pub fn open(path: impl Into<PathBuf>) -> Result<SledKvsEngine> {
        let db = sled::open(path.into())?;
        Ok(SledKvsEngine { db })
    }
}
impl KvsEngine for SledKvsEngine {
    fn set(&mut self, key: String, value: String) -> Result<()> {
        let _result = self.db.insert(key.as_bytes(), value.as_bytes())?;
        Ok(())
    }

    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(result) = self.db.get(key.as_bytes())? {
            return Ok(Some(
                String::from_utf8(result.to_vec())
                    .map_err(|utf8_err| DbError::SledUtf8Error(utf8_err))?,
            ));
        }
        Ok(None)
    }

    fn remove(&mut self, key: String) -> Result<()> {
        match self.db.remove(key.as_bytes()) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => {
                warn!("No such key: {:?}", key);
                Err(DbError::KeyNotFound)
            }
            Err(sled_err) => Err(DbError::SledError(sled_err)),
        }
    }
}
