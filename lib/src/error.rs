//! Error and Result types

use std;
use std::io;

use crate::cli::Action;

#[derive(thiserror::Error, Debug)]
/// Database Error
pub enum DbError {
    /// KvStore accessed before initialization on disk
    #[error("KvStore not initialized. Please initialize using KvStore::open")]
    Uninitialized,
    /// Key not found
    #[error("Key doesn't exist")]
    KeyNotFound,
    /// Offset error
    #[error("Expected action `Set` but found {:?}", _0)]
    OffsetError(Action),
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
    /// Ron SpannedResult error
    #[error("{}", _0)]
    RonSpanned(#[from] ron::error::SpannedError),
    /// Sled Error
    #[error("{}", _0)]
    SledError(#[from] sled::Error),
    /// Sled byte UTF-8 cast failure
    #[error("{}", _0)]
    SledUtf8Error(#[from] std::string::FromUtf8Error),
}

/// KvStore Result type, with error variant representing Database errors
pub type Result<T> = core::result::Result<T, DbError>;
