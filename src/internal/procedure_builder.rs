use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, RwLock},
};

use super::{ProcedureDataType, ProcedureKind};

pub type GlobalData = Arc<RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync + 'static>>>>;

// TODO: Remove `TResolver` and put it into bounds on this type
pub struct UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    pub name: &'static str,
    pub kind: ProcedureKind,
    pub typedef: ProcedureDataType,
    pub data: GlobalData,
    // This can't be generic or a function pointer so boxing is a requirement in stable Rust. It's done at schema-build time so it should be ok.
    // For this to be done without boxing we would need `fn_traits` - https://doc.rust-lang.org/beta/unstable-book/library-features/fn-traits.html
    deref_handler: Box<dyn Fn(TResolver) -> BuiltProcedureBuilder<TResolver>>,
    phantom: PhantomData<TLayerCtx>,
}

impl<TLayerCtx, TResolver> UnbuiltProcedureBuilder<TLayerCtx, TResolver> {
    pub fn new(
        name: &'static str,
        kind: ProcedureKind,
        typedef: ProcedureDataType,
        data: GlobalData,
    ) -> Self {
        Self {
            name,
            kind: kind.clone(),
            typedef: typedef.clone(),
            data: data.clone(),
            // TODO: Make it so this is only boxed in the `Deref` impl so it's a zero cost abstraction!
            deref_handler: Box::new(move |resolver| BuiltProcedureBuilder {
                name,
                kind: kind.clone(),
                typedef: typedef.clone(),
                data: data.clone(),
                resolver,
            }),
            phantom: PhantomData,
        }
    }

    pub fn from_builder<T>(builder: &UnbuiltProcedureBuilder<TLayerCtx, T>) -> Self {
        let (name, kind, typedef, data) = (
            builder.name,
            builder.kind.clone(),
            builder.typedef.clone(),
            builder.data.clone(),
        );

        Self {
            name,
            kind: kind.clone(),
            typedef: typedef.clone(),
            data: data.clone(),
            deref_handler: Box::new(move |resolver| BuiltProcedureBuilder {
                name,
                kind: kind.clone(),
                typedef: typedef.clone(),
                data: data.clone(),
                resolver,
            }),
            phantom: PhantomData,
        }
    }

    pub fn resolver(self, resolver: TResolver) -> BuiltProcedureBuilder<TResolver> {
        BuiltProcedureBuilder {
            name: self.name,
            kind: self.kind,
            typedef: self.typedef,
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
    pub kind: ProcedureKind,
    pub typedef: ProcedureDataType,
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
