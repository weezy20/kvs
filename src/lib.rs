#![allow(unused, warnings)]

use std::{collections::HashMap, hash::Hash};

pub mod cli;
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
pub enum DbError {
    #[error("Key doesn't exist")]
    KeyNotFound,
}

impl<K, V> KvStore<K, V>
where
    K: Eq + Hash,
    V: Clone,
{
    pub fn set(&mut self, key: K, value: V) {
        self.map.insert(key, value);
    }

    pub fn get(&self, key: K) -> Option<V> {
        self.map.get(&key).cloned()
    }

    pub fn remove(&mut self, key: K) {
        let _ = self.map.remove(&key);
    }
}

#[test]
fn foo() {
    assert!(true)
}
