#![deny(missing_docs)]
#![feature(file_create_new)] // https://github.com/rust-lang/rust/pull/98801

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

use log::info;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    hash::Hash,
    path::{Path, PathBuf},
};

pub mod cli;
mod error;
pub use error::{DbError, Result};

/// KvStore implementation
#[derive(Debug, Default)]
pub struct KvStore<K = String, V = String> {
    map: HashMap<K, V>,
    disk: Option<File>,
}

impl<K, V> KvStore<K, V>
where
    K: Eq + Hash,
{
    /// Open on disk KvStore.
    /// On startup, the commands in the log are traversed from oldest to newest, and the in-memory index rebuilt.
    /// When the size of the uncompacted log entries reach a given threshold,
    /// kvs compacts it into a new log, removing redundent entries to reclaim disk space.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore<K, V>> {
        let mut store = KvStore {
            map: HashMap::new(),
            disk: None,
        };
        let wal_path: PathBuf = path.into();
        let opener = |path: &PathBuf| -> Result<File> {
            Ok(OpenOptions::new().read(true).append(true).open(&path)?)
        };
        // If path is a file, load that file
        if wal_path.is_file() {
            let file = opener(&wal_path)?;
            store.disk = Some(file);
        } else if wal_path.is_dir() {
            let wal_path = wal_path.join(Path::new("kv_00001.log"));
            // Try to open default kv_00001.log, or create a file
            match std::fs::metadata(&wal_path) {
                Ok(_) => store.disk = Some(opener(&wal_path)?),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    info!("No kv log found, creating a new one");
                    store.disk = Some(
                        OpenOptions::new()
                            .read(true)
                            .append(true)
                            .create(true)
                            .open(&wal_path)?,
                    );
                    info!("File created and opened successfully: {:?}", wal_path);
                }
                Err(unhandled_err) => return Err(unhandled_err.into()),
            }
        }
        Ok(store)
    }
}

impl<K, V> KvStore<K, V>
where
    K: Clone + Eq + Hash + Into<String>,
    V: Clone + Eq + Into<String>,
{
    /// Set : When setting a key to a value, kvs writes the set command to disk in a sequential log,
    /// then stores the log pointer (file offset) of that command in the in-memory index from key to pointer.
    pub fn set(&mut self, key: K, value: V) -> Result<()> {
        let file = self.disk.as_mut().ok_or(DbError::Uninitialized)?;
        let set_cmd = cli::SetCmd {
            key: key.clone().into(),
            value: value.clone().into(),
        };
        // serialize the set_cmd
        let serialized = format!("SET {}\n", ron::ser::to_string(&set_cmd)?);
        // write serialized to self.disk
        std::io::Write::write_all(file, serialized.as_bytes())?;
        self.map.insert(key, value);
        Ok(())
    }
    /// Get : When retrieving a value for a key with the get command, it searches the index,
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
