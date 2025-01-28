use moka::sync::Cache;

use crate::{store::Value, Store};

pub struct Memory(Cache<String, Value>);

impl Memory {
    pub fn new() -> Self {
        Self(Cache::new(100)) // TODO: Configurable
    }
}

impl Store for Memory {
    fn get(&self, key: &str) -> Option<Value> {
        self.0.get(key).map(|v| v.clone())
    }

    fn set(&self, key: &str, value: Value, ttl: usize) {
        // TODO: Properly set ttl
        self.0.insert(key.to_string(), value);
    }
}
