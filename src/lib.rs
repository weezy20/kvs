pub struct KvStore<K, V>(std::marker::PhantomData<(K, V)>);

pub trait Database<Key, Value> {
    /// set key -> value, mutates existing key
    fn set(&mut self, key: Key, value: Value);
    /// Fetch value
    fn get(&self, key: Key) -> Option<Value>;
    /// Ok indicates successful removal
    fn remove(&mut self, key: Key) -> Result<(), DbError>;
}

#[derive(thiserror::Error, Debug)]
pub enum DbError {
    #[error("Key doesn't exist")]
    KeyNotFound,
}

impl<K, V> Database<K, V> for KvStore<K, V> {
    fn set(&mut self, key: K, value: V) {
        todo!()
    }

    fn get(&self, key: K) -> Option<V> {
        todo!()
    }

    fn remove(&mut self, key: K) -> Result<(), DbError> {
        todo!()
    }
}
