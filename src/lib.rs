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

use log::{info, warn};
use std::{
    collections::HashMap,
    fs::File,
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
    /// Instantiate a new in memory store & disk store on current directory
    /// Note: will overwrite existing file named `kv_wal_00001.log` in current dir
    pub fn new() -> Self {
        let dir: Option<PathBuf> = std::env::current_dir().map_or_else(
            |e| {
                warn!("Cannot instantiate KvStore on current directory: {e:?}");
                None
            },
            Some,
        );
        let disk = if let Some(dir) = dir {
            let file_path = dir.join(format!("kv_wal_{:05}.log", 1));
            // File::create_new is nightly-only API, so this call will overwrite if file exists
            File::create(file_path).ok()
        } else {
            None
        };
        Self {
            map: HashMap::new(),
            disk,
        }
    }
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
        // If path is a file, load that file
        if wal_path.is_file() {
            let file = File::open(&wal_path)?;
            store.disk = Some(file);
        } else if wal_path.is_dir() {
            // Try to open default kv_wal_00001.log, or create a file
            match File::open(&wal_path.join(Path::new("kv_wal_00001.log"))) {
                Ok(f) => store.disk = Some(f),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    info!("No kv_wal_00001.log found, creating a new one");
                    let file_path = wal_path.join(Path::new("kv_wal_00001.log"));
                    let f = File::create(&file_path)?;
                    store.disk = Some(f);
                    info!("File created and opened successfully: {:?}", file_path);
                }
                Err(unhandled_err) => return Err(unhandled_err.into()),
            }
        }
        Ok(store)
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
