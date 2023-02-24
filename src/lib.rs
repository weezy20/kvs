#![allow(unused, warnings)]
#![deny(missing_docs)]

//! Implementation for KvStore:
//! [Find out more here](https://github.com/pingcap/talent-plan/blob/master/courses/rust/projects/project-1/README.md)

use std::{collections::HashMap, hash::Hash};

pub mod cli;
/// KvStore implementation
pub struct KvStore<K, V>
where
    K: Eq + Hash,
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
}
/*
pub trait Database<Key, Value> {
    /// set key -> value, mutates existing key
    fn set(&mut self, key: Key, value: Value);
    /// Fetch value
    fn get(&self, key: Key) -> Option<Value>;
    /// Ok indicates successful removal
    fn remove(&mut self, key: Key) -> Result<(), DbError>;
}
*/

#[allow(unused)]
#[derive(thiserror::Error, Debug)]
/// Database Error
pub enum DbError {
    /// Key not found
    #[error("Key doesn't exist")]
    KeyNotFound,
}

impl<K, V> KvStore<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    /// Set
    pub fn set(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }
    /// Get
    pub fn get(&self, key: K) -> Option<V> {
        self.map.get(&key).cloned()
    }
    /// Remove
    pub fn remove(&mut self, key: K) {
        let _ = self.map.remove(&key);
    }
}

#[test]
fn foo() {
    assert!(true)
}
