use std::{marker::PhantomData, ops::Deref};

pub struct UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    deref_handler: fn(TResolver) -> BuiltProcedureBuilder<TResolver>,
    phantom: PhantomData<TLayerCtx>,
}

impl<TLayerCtx, TResolver> UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    pub(crate) fn new() -> Self {
        Self {
            deref_handler: |resolver| BuiltProcedureBuilder { resolver },
            phantom: PhantomData,
        }
    }

    pub fn resolver(self, resolver: TResolver) -> BuiltProcedureBuilder<TResolver> {
        (self.deref_handler)(resolver)
    }
}

impl<TLayerCtx, TResolver> Deref for UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    type Target = fn(resolver: TResolver) -> BuiltProcedureBuilder<TResolver>;

    fn deref(&self) -> &Self::Target {
        &self.deref_handler
    }
}

pub struct BuiltProcedureBuilder<TResolver> {
    pub(crate) resolver: TResolver,
}
