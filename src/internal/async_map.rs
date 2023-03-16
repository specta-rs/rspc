//! A standard interface for an async map types. This could probs become it's own crate but we can't offer the `get` method using the stdlib API due to lifetime issues, so idk.

mod std_hash_map {
    use std::{collections::HashMap, hash::Hash};

    use super::AsyncMap;

    impl<K, V> AsyncMap<K, V> for HashMap<K, V>
    where
        K: Hash + Eq,
        V: Send + Sync,
    {
        fn contains_key(&self, k: &K) -> bool {
            HashMap::contains_key(self, k)
        }

        fn insert(&mut self, k: K, v: V) -> Option<V> {
            HashMap::insert(self, k, v)
        }

        fn remove(&mut self, k: &K) -> Option<V> {
            HashMap::remove(self, k)
        }
    }
}

pub use std_hash_map::*;

// TODO: Make methods async
// TODO: Add `Mutex<HashMap>` impl

/// TODO
pub trait AsyncMap<K, V> {
    fn contains_key(&self, k: &K) -> bool;
    fn insert(&mut self, k: K, v: V) -> Option<V>;
    fn remove(&mut self, k: &K) -> Option<V>;
}

impl<'a, K, V, T: AsyncMap<K, V>> AsyncMap<K, V> for &'a mut T {
    fn contains_key(&self, k: &K) -> bool {
        T::contains_key(self, k)
    }

    fn insert(&mut self, k: K, v: V) -> Option<V> {
        T::insert(self, k, v)
    }

    fn remove(&mut self, k: &K) -> Option<V> {
        T::remove(self, k)
    }
}
