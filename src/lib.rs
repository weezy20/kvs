#![deny(missing_docs)]

//! Implementation for KvStore:
//! [Find out more here](https://github.com/pingcap/talent-plan/blob/master/courses/rust/projects/project-1/README.md)
//!
//! - *command* - A request or the representation of a request made to the database.
//! These are issued on the command line or over the network.
//! They have an in-memory representation, a textual representation, and a machine-readable serialized representation.
//!
//! - *log* - An on-disk sequence of commands, in the order originally received and executed.
//! Our database's on-disk format is almost entirely made up of logs.
//! It will be simple, but also surprisingly efficient.
//!
//! - *log pointer* - A file offset into the log. Sometimes we'll just call this a "file offset".
//!
//! - *log compaction* - As writes are issued to the database they sometimes invalidate old log entries.
//! For example, writing key/value a = 0 then writing a = 1, makes the first log entry for "a" useless.
//! Compaction — in our database at least — is the process of reducing the size of the database by remove stale commands from the log.
//!
//! - *in-memory index (or index)* - A map of keys to log pointers.
//! When a read request is issued, the in-memory index is searched for the appropriate log pointer,
//! and when it is found the value is retrieved from the on-disk log. In our key/value store, like in bitcask,
//! the index for the entire database is stored in memory.
//!
//! - *index file* - The on-disk representation of the in-memory index.
//! Without this the log would need to be completely replayed to restore the state of the in-memory index each time the database is started.

use std::{collections::HashMap, hash::Hash};

pub mod cli;
/// KvStore implementation
pub struct KvStore<K = String, V = String> {
    map: HashMap<K, V>,
    marker: std::marker::PhantomData<(K, V)>,
}

impl<K, V> KvStore<K, V>
where
    K: Eq + Hash,
{
    /// Instantiate a new in memory store
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            marker: std::marker::PhantomData,
        }
    }
    /// Open on disk KvStore
    pub fn open(_db: &std::path::Path) -> Result<Self> {
        Ok(Self::new())
    }
}

#[derive(thiserror::Error, Debug)]
/// Database Error
pub enum DbError {
    /// Key not found
    #[error("Key doesn't exist")]
    KeyNotFound,
    /// Datbase not found at path
    #[error("Datbase not found at path: {:?}", _0)]
    DatabaseNotFound(&'static std::path::Path)
}

/// KvStore Result type, with error variant representing Database errors
pub type Result<T> = core::result::Result<T, DbError>;

impl<K, V> KvStore<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    /// Set
    pub fn set(&mut self, key: K, value: V) -> Result<()> {
        self.map.insert(key, value);
        Ok(())
    }
    /// Get
    pub fn get(&self, key: K) -> Result<Option<V>> {
        Ok(self.map.get(&key).cloned())
    }
    /// Remove
    pub fn remove(&mut self, key: K) -> Result<()> {
        self.map.remove(&key);
        Ok(())
    }
}
