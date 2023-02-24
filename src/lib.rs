#![allow(unused, warnings)]

pub mod cli;
pub struct KvStore<K, V>(std::marker::PhantomData<(K, V)>);

impl<K, V> KvStore<K, V> {
    pub fn new() -> Self {
        Self(std::marker::PhantomData)
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

#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Key doesn't exist")]
    KeyNotFound,
}

impl<K, V> KvStore<K, V> {
    pub fn set(&mut self, key: K, value: V) {
        unimplemented!()
    }

    pub fn get(&self, key: K) -> Option<V> {
        unimplemented!()
    }

    pub fn remove(&mut self, key: K) -> Result<(), DbError> {
        unimplemented!()
    }
}

#[test]
fn foo() {
    assert!(true)
}