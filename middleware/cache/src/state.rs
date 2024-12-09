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

    pub fn mount(self) -> impl FnOnce(&mut State) {
        let cache = CacheState::builder(Arc::new(self.store));
        move |state: &mut State| {
            state.insert(cache);
        }
    }
}
