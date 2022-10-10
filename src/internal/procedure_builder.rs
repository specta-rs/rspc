use std::{marker::PhantomData, ops::Deref};

use crate::GlobalData;

pub struct UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    data: GlobalData,
    // This can't be generic or a function pointer so boxing is a requirement in stable Rust. It's done at schema-build time so it should be ok.
    // For this to be done without boxing we would need `fn_traits` - https://doc.rust-lang.org/beta/unstable-book/library-features/fn-traits.html
    deref_handler: Box<dyn Fn(TResolver) -> BuiltProcedureBuilder<TResolver>>,
    phantom: PhantomData<TLayerCtx>,
}

impl<TLayerCtx, TResolver> UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    pub fn new(data: GlobalData) -> Self {
        Self {
            data: data.clone(),
            deref_handler: Box::new(move |resolver| BuiltProcedureBuilder {
                data: data.clone(),
                resolver,
            }),
            phantom: PhantomData,
        }
    }

    pub fn resolver(self, resolver: TResolver) -> BuiltProcedureBuilder<TResolver> {
        BuiltProcedureBuilder {
            data: self.data,
            resolver,
        }
    }

    pub fn data(&self) -> GlobalData {
        self.data.clone()
    }
}

impl<TLayerCtx, TResolver> Deref for UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    type Target = Box<dyn Fn(TResolver) -> BuiltProcedureBuilder<TResolver>>;

    fn deref(&self) -> &Self::Target {
        &self.deref_handler
    }
}

pub struct BuiltProcedureBuilder<TResolver> {
    pub data: GlobalData,
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
