use std::{any::Any, sync::Arc};

use moka::sync::Cache;

pub trait Store: Send + Sync + 'static {
    // fn get<V: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<V>;
    // fn set<V: Clone + Send + Sync + 'static>(&self, key: &str, value: &V, ttl: usize);
}

impl<S: Store + Send> Store for Arc<S> {
    // fn get<V: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<V> {
    //     self.as_ref().get(key)
    // }

    // fn set<V: Clone + Send + Sync + 'static>(&self, key: &str, value: &V, ttl: usize) {
    //     self.as_ref().set(key, value, ttl)
    // }
}

struct Value(Box<dyn DynClone + Send + Sync>);

impl Clone for Value {
    fn clone(&self) -> Self {
        Self(self.0.dyn_clone())
    }
}

// TODO: Sealing this better.
pub trait DynClone: Send + Sync {
    // Return `Value` instead of `Box` directly for sealing
    fn dyn_clone(&self) -> Box<dyn DynClone + Send + Sync>;
}
impl<T: Clone + Send + Sync + 'static> DynClone for T {
    fn dyn_clone(&self) -> Box<dyn DynClone + Send + Sync> {
        Box::new(self.clone())
    }
}

pub struct Memory(Cache<String, Value>);

impl Memory {
    pub fn new() -> Self {
        Self(Cache::new(100)) // TODO: Configurable
    }
}

impl Store for Memory {
    // fn get<V: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<V> {
    //     self.0.get(key).map(|v| v.downcast_ref().clone())
    // }

    // fn set<V: Clone + Send + Sync + 'static>(&self, key: &str, value: &V, ttl: usize) {
    //     todo!()
    // }
}
