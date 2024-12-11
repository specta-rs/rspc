use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    hash::{BuildHasherDefault, Hasher},
};

/// A hasher for `TypeId`s that takes advantage of its known characteristics.
///
/// Author of `anymap` crate has done research on the topic:
/// https://github.com/chris-morgan/anymap/blob/2e9a5704/src/lib.rs#L599
#[derive(Debug, Default)]
struct NoOpHasher(u64);

impl Hasher for NoOpHasher {
    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!("This NoOpHasher can only handle u64s")
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

pub struct State(
    HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>, BuildHasherDefault<NoOpHasher>>,
);

impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("State").field(&self.0.keys()).finish()
    }
}

impl Default for State {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl State {
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.0.get(&TypeId::of::<T>()).map(|v| {
            v.downcast_ref::<T>()
                .expect("unreachable: TypeId matches but downcast failed")
        })
    }

    pub fn get_mut<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.0.get(&TypeId::of::<T>()).map(|v| {
            v.downcast_ref::<T>()
                .expect("unreachable: TypeId matches but downcast failed")
        })
    }

    pub fn get_or_init<T: Send + Sync + 'static>(&mut self, init: impl FnOnce() -> T) -> &T {
        self.0
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(init()))
            .downcast_ref::<T>()
            .expect("unreachable: TypeId matches but downcast failed")
    }

    pub fn get_mut_or_init<T: Send + Sync + 'static>(
        &mut self,
        init: impl FnOnce() -> T,
    ) -> &mut T {
        self.0
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(init()))
            .downcast_mut::<T>()
            .expect("unreachable: TypeId matches but downcast failed")
    }

    pub fn contains_key<T: Send + Sync + 'static>(&self) -> bool {
        self.0.contains_key(&TypeId::of::<T>())
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, t: T) {
        self.0.insert(TypeId::of::<T>(), Box::new(t));
    }

    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.0.remove(&TypeId::of::<T>()).map(|v| {
            *v.downcast::<T>()
                .expect("unreachable: TypeId matches but downcast failed")
        })
    }
}
