//! Error and Result types

use std::io;

use std;

#[derive(thiserror::Error, Debug)]
/// Database Error
pub enum DbError {
    /// Key not found
    #[error("Key doesn't exist")]
    KeyNotFound,
    /// Datbase not found at path
    #[error("Datbase not found at path: {:?}", _0)]
    DatabaseNotFound(std::path::PathBuf),
    /// Io Error
    #[error("{}", _0)]
    Io(#[from] io::Error),
    /// Serde Error
    #[error("{}", _0)]
    Serde(#[from] serde_json::Error),
}

/// KvStore Result type, with error variant representing Database errors
pub type Result<T> = core::result::Result<T, DbError>;
