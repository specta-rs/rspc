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
    procedure::AlphaProcedure, AlphaBaseMiddleware, AlphaMiddlewareBuilder,
    AlphaMiddlewareBuilderLike, AlphaMiddlewareLayerBuilder, AlphaMiddlewareLike, AlphaRouter,
    MiddlewareArgMapper, MiddlewareMerger, MissingResolver, ResolverFunction,
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
    pub fn with<TNewMiddleware>(
        self,
        builder: impl Fn(AlphaMiddlewareBuilder<TCtx, (), ()>) -> TNewMiddleware, // TODO: Remove builder closure
    ) -> AlphaProcedure<
        MissingResolver<TNewMiddleware::NewCtx>,
        (),
        AlphaMiddlewareLayerBuilder<AlphaBaseMiddleware<TCtx>, TNewMiddleware>,
    >
    where
        TNewMiddleware: AlphaMiddlewareLike<LayerCtx = TCtx> + Send + Sync + 'static,
    {
        let mw = builder(AlphaMiddlewareBuilder(PhantomData));
        AlphaProcedure::new_from_middleware(AlphaMiddlewareLayerBuilder {
            middleware: AlphaBaseMiddleware::new(),
            mw,
        })
    }
    pub fn query<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, RMarker, AlphaBaseMiddleware<TCtx>>
    where
        R: ResolverFunction<RMarker, LayerCtx = TCtx> + Fn(TCtx, R::Arg) -> R::Result,
    {
        AlphaProcedure::new_from_resolver(ProcedureKind::Query, AlphaBaseMiddleware::new(), builder)
    }

    pub fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, RMarker, AlphaBaseMiddleware<TCtx>>
    where
        R: ResolverFunction<RMarker, LayerCtx = TCtx> + Fn(TCtx, R::Arg) -> R::Result,
    {
        AlphaProcedure::new_from_resolver(
            ProcedureKind::Mutation,
            AlphaBaseMiddleware::new(),
            builder,
        )
    }
}
