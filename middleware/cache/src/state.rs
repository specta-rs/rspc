use std::sync::Arc;

use rspc::State;

use crate::Store;

pub struct CacheState<S = Arc<dyn Store>> {
    store: S,
}

impl<S: Store> CacheState<S> {
    pub fn builder(store: S) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &S {
        &self.store
    }

    // TODO: Default ttl

    pub fn mount(self) -> impl FnOnce(&mut State) {
        let cache: Arc<dyn Store> = Arc::new(self.store);
        let cache: CacheState<Arc<dyn Store>> = CacheState::<Arc<dyn Store>>::builder(cache);
        move |state: &mut State| {
            println!("SETUP"); // TODO
            state.insert::<CacheState<Arc<dyn Store>>>(cache);
        }
    }
}
