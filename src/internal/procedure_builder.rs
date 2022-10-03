use std::{marker::PhantomData, ops::Deref};

use crate::GlobalData;

pub struct UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    data: GlobalData,
    deref_handler: fn(TResolver) -> BuiltProcedureBuilder<TResolver>,
    phantom: PhantomData<TLayerCtx>,
}

impl<TLayerCtx, TResolver> UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    pub fn new(data: GlobalData) -> Self {
        Self {
            data,
            deref_handler: |resolver| BuiltProcedureBuilder {
                data: None,
                resolver,
            },
            phantom: PhantomData,
        }
    }

    pub fn resolver(self, resolver: TResolver) -> BuiltProcedureBuilder<TResolver> {
        BuiltProcedureBuilder {
            data: Some(self.data),
            resolver,
        }
    }
}

impl<TLayerCtx, TResolver> Deref for UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    type Target = fn(resolver: TResolver) -> BuiltProcedureBuilder<TResolver>;

    fn deref(&self) -> &Self::Target {
        &self.deref_handler
    }
}

pub struct BuiltProcedureBuilder<TResolver> {
    // TODO: This shouldn't be an option
    pub data: Option<GlobalData>,
    pub resolver: TResolver,
}

impl<TResolver> BuiltProcedureBuilder<TResolver> {
    pub fn map<TOutResolver>(
        self,
        func: impl Fn(Self) -> BuiltProcedureBuilder<TOutResolver>,
    ) -> BuiltProcedureBuilder<TOutResolver> {
        func(self)
    }
}
