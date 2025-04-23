use super::{DbError, Result};
use bytes::Bytes;
use log::{error, info, warn};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    sync::atomic::AtomicUsize,
};
/// (Sequence, ByteOffset, Record length)
#[derive(Debug, Clone, Copy)]
pub struct Offset {
    /// Sequence number of the datafile
    sequence: usize,
    /// Byte offset of the record in the datafile
    byte_offset: usize,
    /// Length of the record in bytes
    record_length: usize,
}
impl Default for Offset {
    fn default() -> Self {
        Offset {
            sequence: 0,
            byte_offset: 0,
            record_length: 0,
        }
    }
}
type Key = Bytes;
/// KvsStore database
#[derive(Debug)]
pub struct KvsDatabase {
    /// Directory to use as KvsStore database
    pub path: PathBuf,
    /// List of datafiles in the database
    datafiles: AtomicUsize,
    /// In memory cache
    pub(crate) map: HashMap<Key, Offset>,
}

impl Default for KvsDatabase {
    fn default() -> Self {
        let tmp_or_current = env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
            .join("kvs_store");

        match Self::open(&tmp_or_current) {
            Ok(db) => db,
            Err(err) => {
                error!("Cannot create default KvsDatabase : {err}");
                panic!(
                    "Failed to open default database at {}",
                    tmp_or_current.display()
                );
            }
        }
    }
}

impl KvsDatabase {
    /// Create a new KvsDatabase
    pub fn open(path: &Path) -> Result<Self> {
        let db = Self {
            path: path.to_path_buf(),
            datafiles: AtomicUsize::new(0),
            map: HashMap::new(),
        };
        if !path.try_exists()? {
            info!(
                "Database directory does not exist, creating: {}",
                path.display()
            );
            std::fs::create_dir_all(path).unwrap_or_else(|_| {
                panic!("Failed to create database directory: {}", path.display())
            });
        }
        // Read directory entries in parallel
        let entries: Vec<_> = match std::fs::read_dir(path) {
            Ok(read_dir) => read_dir.filter_map(|e| e.ok()).collect(),
            Err(_) => {
                warn!("Failed to read directory entries");
                warn!("Instantiating new KvsDatabase");
                vec![]
            }
        };
        entries.par_iter().for_each(|entry| {
            if let Ok(path_exists) = entry.path().try_exists() {
                if path_exists {
                    if let Ok(_seq) = KvsDatafile::validate(&path) {
                        db.datafiles
                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        info!("Found datafile: {}", path.display());
                    } else {
                        warn!("Invalid datafile name: {}", path.display());
                    }
                }
            }
        });

        Ok(db)
    }
}

/// Datafile representation
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct KvsDatafile(u64);

impl KvsDatafile {
    /// Instantiate a new DatafileName using a sequence number
    pub fn file_name(sequence: u64) -> String {
        format!("kv_{sequence}.data")
    }
    /// Create a new DatafileName sequence using current DatafileName sequence + 1
    pub fn next(&self) -> Result<Self> {
        Ok(KvsDatafile(self.0.saturating_add(1)))
    }
    /// Get sequence number from DatafileName
    pub fn validate(path: &Path) -> Result<u64> {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .ok_or_else(|| DbError::InvalidDatafileName(path.to_string_lossy().into_owned()))?;

        if let Some(seq_str) = file_name
            .strip_prefix("kv_")
            .and_then(|s| s.strip_suffix(".data"))
        {
            seq_str.parse::<u64>().map_err(|e| {
                DbError::InvalidDatafileName(format!("{}: parse error - {}", file_name, e))
            })
        } else {
            Err(DbError::InvalidDatafileName(file_name.to_string()))
        }
    }
}
