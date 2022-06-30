use std::collections::BTreeMap;

use crate::{KeyDefinition, MiddlewareChainBase};

/// TODO
pub(crate) struct Operation<TOperationKey, TCtx>
where
    TOperationKey: KeyDefinition,
{
    name: &'static str, // TODO: move this to a const generic when support for that is added
    operations: BTreeMap<TOperationKey::KeyRaw, MiddlewareChainBase<TCtx>>,
}

impl<TOperationKey, TCtx> Operation<TOperationKey, TCtx>
where
    TOperationKey: KeyDefinition,
    TCtx: 'static,
{
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            operations: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, key: TOperationKey::KeyRaw, handler: MiddlewareChainBase<TCtx>) {
        if self.operations.contains_key(&key) {
            panic!(
                "rspc error: operation '{}' already has resolver with name {:?}",
                self.name, key
            );
        }

        self.operations.insert(key, Box::new(handler));
    }

    pub fn get(&self, key: TOperationKey::KeyRaw) -> Option<&MiddlewareChainBase<TCtx>> {
        self.operations.get(&key)
    }

    pub fn operations(self) -> BTreeMap<TOperationKey::KeyRaw, MiddlewareChainBase<TCtx>> {
        self.operations
    }
}
