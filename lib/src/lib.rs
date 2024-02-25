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

use lazy_static::lazy_static;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use ron::ser::PrettyConfig;
use serde::Deserialize;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Seek, Write},
    path::{Path, PathBuf},
};

pub mod cli;
mod error;
mod utils;
pub use error::{DbError, Result};
pub use utils::*;

use crate::cli::{Action, RmCmd, SetCmd};

lazy_static! {
    static ref RON_CONFIG: PrettyConfig = PrettyConfig::default()
        .depth_limit(0)
        .struct_names(false)
        .separate_tuple_members(false);
}

/// Backend for KvStore
pub trait KvsEngine {
    /// Set key to value
    fn set(&mut self, key: String, value: String) -> Result<()>;
    /// Query for key
    fn get(&self, key: String) -> Result<Option<String>>;
    /// Remove key
    fn remove(&mut self, key: String) -> Result<()>;
}

/// File offset
pub type Offset = u64;
/// KvStore implementation
#[derive(Debug, Default)]
pub struct KvStore {
    /// In memory index from key -> offset in log
    pub(crate) map: HashMap<String, Offset>,
    pub(crate) disk: Option<RefCell<File>>,
    pub(crate) offset: Offset,
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
            offset: Default::default(),
        };
        // -- Load log file into KvStore --
        let wal_path: PathBuf = path.into();
        let opener = |path: &PathBuf| -> Result<RefCell<File>> {
            Ok(RefCell::new(
                OpenOptions::new().read(true).append(true).open(&path)?,
            ))
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
                    store.disk = Some(RefCell::new(
                        OpenOptions::new()
                            .read(true)
                            .append(true)
                            .create(true)
                            .open(&wal_path)?,
                    ));
                    info!("File created and opened successfully: {:?}", wal_path);
                }
                Err(unhandled_err) => return Err(unhandled_err.into()),
            }
        }
        assert!(store.disk.is_some());
        let mut disk = store
            .disk
            .as_ref()
            .expect("Checked above | cannot fail")
            .borrow_mut();
        // -- Initialize the memory map with disk commands --
        // Check if a in memory index is already built, if yes, use that :
        let mem_idx = std::env::current_dir().unwrap().join("kv_memory.index");
        if mem_idx.exists() {
            debug!("Loading in memory index from file {mem_idx:?}");
            match ron::from_str(std::fs::read_to_string(&mem_idx).unwrap().as_str()) {
                Ok(map) => {
                    store.map = map;
                    store.offset =
                        (BufReader::new(disk.try_clone()?).lines().count() + 1) as Offset;
                    debug!("Loaded in memory index with offset {}", store.offset);
                }
                Err(err) => error!("Cannot load in memory index: {:?}", err),
            }
        } else {
            let log = read_action_from_log(&mut disk)?;
            // -- Replay the commands in the log --
            let mut offset: Offset = 0;
            for (idx, action) in log.iter().enumerate() {
                // Offset is just tracking the current deserialized index of Action in the list of Actions
                // Offset numbering begins at 1. Offset of 1 = first line of the log
                offset = (idx + 1) as Offset;
                match action {
                    Action::Set(SetCmd { key, .. }) => {
                        store.map.insert(key.clone(), offset);
                    }
                    Action::Get(_) => {
                        /* Idempotent action.
                        We should not increase offset on a Get since we ensure we never write a Get to a log */
                    }
                    Action::Remove(RmCmd { key }) => {
                        if let Some(_rm_offset) = store.map.remove(key) {}
                    }
                };
            }
            // Store latest (empty) offset in KvStore for future insertions
            store.offset = offset + 1;
            debug!(
                "KvStore initialized without in-mem index and offset {}",
                store.offset
            );
        }
        drop(disk);
        Ok(store)
    }

    /// Run compaction on the disk log
    pub fn compaction(&mut self) -> Result<()> {
        let mut file = self
            .disk
            .as_ref()
            .ok_or(DbError::Uninitialized)?
            .borrow_mut()
            .try_clone()?;
        file.rewind()?;

        let log: Vec<Action> = read_action_from_log(&mut file)?;
        // Hold unique keys last set value, None in case it was removed
        let mut unique_keys: BTreeMap<String, Option<String>> = BTreeMap::new();
        let mut compacted_log: Vec<Action> = Vec::with_capacity(log.capacity());
        // debug!("Log to compact : {log:?}");
        log.iter().rev().for_each(|action| {
            // debug!("compact OFFSET: {offset}");
            match action {
                Action::Set(SetCmd { key, value }) => {
                    if !unique_keys.contains_key(key) {
                        unique_keys.insert(key.to_string(), Some(value.to_string()));
                    }
                }
                Action::Get(_) => (),
                Action::Remove(RmCmd { key }) => {
                    if !unique_keys.contains_key(key) {
                        unique_keys.insert(key.to_string(), None);
                    }
                }
            }
        });
        debug!("Unique Keys : {:?}", unique_keys);
        // Rebuild log
        compacted_log.extend(unique_keys.into_iter().rev().map(|(key, value)| {
            if let Some(value) = value {
                Action::Set(SetCmd { key, value })
            } else {
                Action::Remove(RmCmd { key })
            }
        }));
        // Recompute offsets
        self.map.clear();
        let mut offset: Offset = 1;
        compacted_log.iter().for_each(|action| {
            match action {
                Action::Set(SetCmd { key, .. }) => {
                    self.map.insert(key.clone(), offset);
                }
                Action::Get(_) => {}
                Action::Remove(RmCmd { key }) => {
                    self.map.remove(key);
                }
            };
            offset += 1;
        });
        self.offset = offset;
        debug!("Post compaction, current offset {}", self.offset);
        // Write compacted_log to self.disk
        let mut file = self
            .disk
            .as_ref()
            .ok_or(DbError::Uninitialized)?
            .borrow_mut();
        file.rewind()?;
        // Clear file contents
        file.set_len(0)?;
        // Write serialized to file but one entry at a time instead of as a Vec
        for action in compacted_log {
            let serialized = ron::ser::to_string_pretty(&action, RON_CONFIG.to_owned())? + "\n";
            file.write_all(serialized.as_bytes())?;
        }
        Ok(())
    }
}
/// Deserialize on disk log
fn read_action_from_log(disk: &mut File) -> Result<Vec<Action>> {
    let mut buf = String::new();
    let _bytes_read = (*disk).read_to_string(&mut buf).map_err(|e| {
        error!("Cannot load log file into memory");
        e
    })?;
    let mut de = ron::Deserializer::from_str(&buf).expect("RON: deserializer init error");
    let log: Vec<Action> = std::iter::from_fn({
        move || {
            de.end()
                .is_err()
                .then_some(Action::deserialize::<_>(&mut de))
        }
    })
    .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(log)
}

impl KvsEngine for KvStore {
    /// Set : When setting a key to a value, kvs writes the set command to disk in a sequential log,
    /// then stores the log pointer (file offset) of that command in the in-memory index from key to pointer.
    fn set(&mut self, key: String, value: String) -> Result<()> {
        if self.map.len() > 500 {
            // trigger compaction
            (*self).compaction()?;
        }
        let mut file = self
            .disk
            .as_ref()
            .ok_or(DbError::Uninitialized)?
            .borrow_mut();
        self.map.insert(key.clone(), self.offset);
        self.offset += 1;
        let set_cmd = Action::Set(cli::SetCmd {
            key: key.into(),
            value: value.into(),
        });
        // serialize the set_cmd
        let serialized = ron::ser::to_string_pretty(&set_cmd, RON_CONFIG.to_owned())? + "\n";
        // let serialized = ron::ser::to_string(&set_cmd)? + "\n";
        // write serialized to self.disk
        // TODO : Maybe think about optimizing this? file sys-call on every set cmd?
        file.write_all(serialized.as_bytes())?;

        Ok(())
    }
    /// Get : When retrieving a value for a key with the get command, it searches the index,
    /// and if found then loads from the log the command at the corresponding log pointer,
    /// evaluates the command and returns the result.
    fn get(&self, key: String) -> Result<Option<String>> {
        if let Some(&offset) = self.map.get(&key) {
            debug!("GET offset: {:?}", offset);
            // File reset seek on self.disk
            let mut file = self
                .disk
                .as_ref()
                .ok_or(DbError::Uninitialized)?
                .borrow_mut()
                .try_clone()?;
            file.rewind()?;
            // Read file line number offset
            let buf = BufReader::new(file)
                .lines()
                .enumerate()
                .map(|(num, line)| {
                    log::debug!("LOG line: {:?}", line.as_ref());
                    line.expect(&format!(
                        "Log Error : Failed to read LOG line number {}",
                        num + 1
                    ))
                })
                .nth(offset.saturating_sub(1) as usize)
                .expect("Offset contents cannot be empty");
            let set_cmd: Action = ron::de::from_str(&buf)?;
            // Retain in memory idx if not already present in current working directory
            let mem_idx = std::env::current_dir().unwrap().join("kv_memory.index");
            if mem_idx.exists() {
                let file = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(mem_idx)?;
                let mut file = BufWriter::new(file);
                // Write contents of in memory map to file
                file.write(ron::to_string(&self.map)?.as_bytes())?;
            }
            match set_cmd {
                Action::Set(set_cmd) => Ok(Some(set_cmd.value)),
                action => Err(DbError::OffsetError(action)),
            }
        } else {
            Ok(None)
        }
    }
    /// Remove : When removing a key, similarly, kvs writes the rm command in the log,
    /// Checking to see first that the key exists
    /// then removes the key from the in-memory index.
    fn remove(&mut self, key: String) -> Result<()> {
        // Check using in memory map
        if self.map.contains_key(&key) {
            let mut file = self
                .disk
                .as_ref()
                .ok_or(DbError::Uninitialized)?
                .borrow_mut();
            let rm_cmd = Action::Remove(cli::RmCmd { key: key.clone() });
            // serialize the rm_cmd
            let serialized = ron::ser::to_string_pretty(&rm_cmd, RON_CONFIG.to_owned())? + "\n";
            // let serialized = ron::ser::to_string(&rm_cmd)? + "\n";
            // write serialized to self.disk
            // TODO : Maybe think about optimizing this? file sys-call on every set cmd?
            file.write_all(serialized.as_bytes())?;
            self.map.remove(&key);
            Ok(())
        } else {
            error!("No such key: {:?}", key);
            Err(DbError::KeyNotFound)
        }
    }
}
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
        unreachable!("Success or UTF-8 cast fails and returns an error");
    }

    fn remove(&mut self, key: String) -> Result<()> {
        let result = self.db.remove(key.as_bytes()).map(|opt| {
            if opt.is_none() {
                error!("No such key: {:?}", key);
            }
            // We intentionally get rid of the value we just removed
            ()
        });
        Ok(result?)
    }
}
