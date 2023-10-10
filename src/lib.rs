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

use log::info;
use ron::ser::PrettyConfig;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
};

pub mod cli;
mod error;
pub use error::{DbError, Result};

use crate::cli::{Action, RmCmd, SetCmd};

/// KvStore implementation
#[derive(Debug, Default)]
pub struct KvStore {
    map: HashMap<String, String>,
    disk: Option<File>,
}

impl KvStore {
    /// Open on disk KvStore.
    /// On startup, the commands in the log are traversed from oldest to newest, and the in-memory index rebuilt.
    /// When the size of the uncompacted log entries reach a given threshold,
    /// kvs compacts it into a new log, removing redundent entries to reclaim disk space.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
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
        assert!(store.disk.is_some());
        // Initialize the memory map with disk commands:
        let mut d = store.disk.as_ref().unwrap();
        let mut buf = String::new();
        let _bytes_read = d.read_to_string(&mut buf)?;
        let v = std::iter::from_fn({
            let mut de = ron::Deserializer::from_str(&buf).unwrap();
            move || {
                de.end()
                    .is_err()
                    .then_some(Action::deserialize::<_>(&mut de))
            }
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
        for action in v {
            let _ = match action {
                Action::Set(SetCmd { key, value }) => store.map.insert(key, value),
                Action::Get(_) => None,
                Action::Rm(RmCmd { key }) => store.map.remove(&key),
            };
        }
        Ok(store)
    }
}

impl KvStore {
    /// Set : When setting a key to a value, kvs writes the set command to disk in a sequential log,
    /// then stores the log pointer (file offset) of that command in the in-memory index from key to pointer.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.map.insert(key.clone(), value.clone());
        let file = self.disk.as_mut().ok_or(DbError::Uninitialized)?;
        let set_cmd = cli::SetCmd {
            key: key.into(),
            value: value.into(),
        };
        // serialize the set_cmd
        let ron_config = PrettyConfig::default().struct_names(true);
        let serialized = ron::ser::to_string_pretty(&set_cmd, ron_config)? + "\n";
        // write serialized to self.disk
        // TODO : Maybe think about optimizing this? file sys-call on every set cmd?
        std::io::Write::write_all(file, serialized.as_bytes())?;
        Ok(())
    }
    /// Get : When retrieving a value for a key with the get command, it searches the index,
    /// and if found then loads from the log the command at the corresponding log pointer,
    /// evaluates the command and returns the result.
    pub fn get(&self, key: String) -> Result<Option<String>> {
        Ok(self.map.get(&key).cloned())
    }
    /// Remove : When removing a key, similarly, kvs writes the rm command in the log,
    /// Checking to see first that the key exists
    /// then removes the key from the in-memory index.
    pub fn remove(&mut self, key: String) -> Result<()> {
        self.map.remove(&key);
        // Check using in memory map
        if self.map.contains_key(&key) {
            let file = self.disk.as_mut().ok_or(DbError::Uninitialized)?;
            let rm_cmd = cli::RmCmd { key: key.into() };
            // serialize the rm_cmd
            let ron_config = PrettyConfig::default().struct_names(true);
            let serialized = ron::ser::to_string_pretty(&rm_cmd, ron_config)? + "\n";
            // write serialized to self.disk
            // TODO : Maybe think about optimizing this? file sys-call on every set cmd?
            std::io::Write::write_all(file, serialized.as_bytes())?;
        }
        Ok(())
    }
}
