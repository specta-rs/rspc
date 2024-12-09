use std::{any::Any, sync::Arc};

pub trait Store: Send + Sync + 'static {
    fn get(&self, key: &str) -> Option<Value>;

    fn set(&self, key: &str, value: Value, ttl: usize);
}

impl Store for Arc<dyn Store> {
    fn get(&self, key: &str) -> Option<Value> {
        self.as_ref().get(key)
    }

    fn set(&self, key: &str, value: Value, ttl: usize) {
        self.as_ref().set(key, value, ttl)
    }
}

impl<S: Store + Send> Store for Arc<S> {
    fn get(&self, key: &str) -> Option<Value> {
        self.as_ref().get(key)
    }

    fn set(&self, key: &str, value: Value, ttl: usize) {
        self.as_ref().set(key, value, ttl)
    }
}

pub struct Value(Box<dyn Repr + Send + Sync>);

impl Value {
    pub fn new<T: Clone + Send + Sync + 'static>(v: T) -> Self {
        Self(Box::new(v))
    }

    pub fn downcast_ref<T: Clone + Send + Sync + 'static>(&self) -> Option<&T> {
        self.0.inner().downcast_ref()
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        Self(self.0.dyn_clone())
    }
}

// TODO: Sealing this better.
trait Repr: Send + Sync + 'static {
    // Return `Value` instead of `Box` directly for sealing
    fn dyn_clone(&self) -> Box<dyn Repr + Send + Sync>;

    fn inner(&self) -> &dyn Any;
}
impl<T: Clone + Send + Sync + 'static> Repr for T {
    fn dyn_clone(&self) -> Box<dyn Repr + Send + Sync> {
        Box::new(self.clone())
    }

    fn inner(&self) -> &dyn Any {
        self
    }
}
