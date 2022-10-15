use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, RwLock},
};

pub type GlobalData = Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>>>;

pub struct UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    pub name: &'static str,
    pub data: GlobalData,
    // This can't be generic or a function pointer so boxing is a requirement in stable Rust. It's done at schema-build time so it should be ok.
    // For this to be done without boxing we would need `fn_traits` - https://doc.rust-lang.org/beta/unstable-book/library-features/fn-traits.html
    deref_handler: Box<dyn Fn(TResolver) -> BuiltProcedureBuilder<TResolver>>,
    phantom: PhantomData<TLayerCtx>,
}

impl<TLayerCtx, TResolver> UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    pub fn new(name: &'static str, data: GlobalData) -> Self {
        Self {
            name,
            data: data.clone(),
            // TODO: Make it so this is only boxed in the `Deref` impl so it's a zero cost abstraction!
            deref_handler: Box::new(move |resolver| BuiltProcedureBuilder {
                name,
                data: data.clone(),
                resolver,
            }),
            phantom: PhantomData,
        }
    }

    pub fn resolver(self, resolver: TResolver) -> BuiltProcedureBuilder<TResolver> {
        BuiltProcedureBuilder {
            name: self.name,
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
    pub name: &'static str,
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
