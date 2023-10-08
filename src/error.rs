//! Error and Result types

use std::io;
use std;

#[derive(thiserror::Error, Debug)]
/// Database Error
pub enum DbError {
    /// KvStore accessed before initialization on disk
    #[error("KvStore not initialized. Please initialize using KvStore::Open")]
    Uninitialized,
    /// Key not found
    #[error("Key doesn't exist")]
    KeyNotFound,
    /// Datbase not found at path
    #[error("Datbase not found at path: {:?}", _0)]
    DatabaseNotFound(std::path::PathBuf),
    /// Io Error
    #[error("{}", _0)]
    Io(#[from] io::Error),
    /// SerdeJson Error
    #[error("{}", _0)]
    SerdeJson(#[from] serde_json::Error),
    /// SerdeRon Error
    #[error("{}", _0)]
    SerdeRon(#[from] ron::Error),
}

/// KvStore Result type, with error variant representing Database errors
pub type Result<T> = core::result::Result<T, DbError>;
