use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use specta::Type;

use crate::{
    impl_procedure_like,
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, MiddlewareLayerBuilder, ProcedureKind,
        UnbuiltProcedureBuilder,
    },
    MiddlewareBuilder, MiddlewareLike, RequestLayer, RouterBuilder,
};

use super::{
    AlphaBaseMiddleware, AlphaMiddlewareBuilder, AlphaMiddlewareLayerBuilder, AlphaMiddlewareLike,
    AlphaRouter, MissingResolver, ResolverFunction,
};

pub struct Rspc<
    TCtx = (), // The is the context the current router was initialised with
> where
    TCtx: Send + Sync + 'static,
{
    builders: Vec<Box<dyn FnOnce()>>,
    phantom: PhantomData<TCtx>,
}

#[allow(clippy::new_without_default)]
impl<TCtx> Rspc<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    pub const fn new() -> Self {
        Self {
            builders: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<TCtx> Rspc<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    pub fn router(&self) -> AlphaRouter<TCtx> {
        AlphaRouter::new()
    }

    // TODO: Remove the `BaseMiddleware` from this join cause it shouldn't be required
    pub fn with<TNewLayerCtx, TNewMiddleware>(
        self,
        builder: impl Fn(AlphaMiddlewareBuilder<TCtx>) -> TNewMiddleware, // TODO: Remove builder closure
    ) -> crate::alpha::procedure::AlphaProcedure<
        TCtx,
        TNewLayerCtx,
        MissingResolver<TNewLayerCtx>,
        (),
        (),
        AlphaMiddlewareLayerBuilder<
            TCtx,
            TCtx,
            TNewLayerCtx,
            AlphaBaseMiddleware<TCtx>,
            TNewMiddleware,
        >,
    >
    where
        TNewLayerCtx: Send + Sync + 'static,
        TNewMiddleware: AlphaMiddlewareLike<TCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
    {
        let mw = builder(AlphaMiddlewareBuilder(PhantomData));
        crate::alpha::procedure::AlphaProcedure::new_from_middleware(AlphaMiddlewareLayerBuilder {
            middleware: AlphaBaseMiddleware::new(),
            mw,
            phantom: PhantomData,
        })
    }

    impl_procedure_like!();
}
