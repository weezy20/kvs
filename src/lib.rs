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

use std::{collections::HashMap, hash::Hash, path::PathBuf};

pub mod cli;
mod error;
pub use error::{DbError, Result};

/// KvStore implementation
pub struct KvStore<K = String, V = String>
{
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
    /// Open on disk KvStore.
    /// On startup, the commands in the log are traversed from oldest to newest, and the in-memory index rebuilt.
    /// When the size of the uncompacted log entries reach a given threshold,
    /// kvs compacts it into a new log, removing redundent entries to reclaim disk space.
    pub fn open(_db: impl Into<PathBuf>) -> Result<KvStore<K, V>> {
        // TODO
        Ok(Self::new())
    }
}

impl<K, V> Default for KvStore<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> KvStore<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    /// Set : When setting a key to a value, kvs writes the set command to disk in a sequential log,
    /// then stores the log pointer (file offset) of that command in the in-memory index from key to pointer.
    pub fn set(&mut self, key: K, value: V) -> Result<()> {
        self.map.insert(key, value);
        Ok(())
    }
    /// Get : . When retrieving a value for a key with the get command, it searches the index,
    /// and if found then loads from the log the command at the corresponding log pointer,
    /// evaluates the command and returns the result.
    pub fn get(&self, key: K) -> Result<Option<V>> {
        Ok(self.map.get(&key).cloned())
    }
    /// Remove : When removing a key, similarly, kvs writes the rm command in the log,
    /// then removes the key from the in-memory index.
    pub fn remove(&mut self, key: K) -> Result<()> {
        self.map.remove(&key);
        Ok(())
    }
}
