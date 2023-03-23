use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use specta::Type;

use crate::{
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, MiddlewareLayerBuilder, ProcedureKind,
        UnbuiltProcedureBuilder,
    },
    MiddlewareBuilder, MiddlewareLike, RequestLayer, RouterBuilder, StreamRequestLayer,
};

use super::{
    procedure::AlphaProcedure, AlphaBaseMiddleware, AlphaMiddlewareBuilder,
    AlphaMiddlewareBuilderLike, AlphaMiddlewareLayerBuilder, AlphaMiddlewareLike, AlphaRouter,
    MiddlewareArgMapper, MissingResolver, RequestKind, RequestLayerMarker, ResolverFunction,
    StreamLayerMarker,
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
        TNewMiddleware: AlphaMiddlewareLike<LayerCtx = TCtx>,
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
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, AlphaBaseMiddleware<TCtx>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>,
    {
        AlphaProcedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Query),
            AlphaBaseMiddleware::new(),
            builder,
        )
    }

    pub fn mutation<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, RequestLayerMarker<RMarker>, AlphaBaseMiddleware<TCtx>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>,
    {
        AlphaProcedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Mutation),
            AlphaBaseMiddleware::new(),
            builder,
        )
    }

    pub fn subscription<R, RMarker>(
        self,
        builder: R,
    ) -> AlphaProcedure<R, StreamLayerMarker<RMarker>, AlphaBaseMiddleware<TCtx>>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: StreamRequestLayer<R::RequestMarker>,
    {
        AlphaProcedure::new_from_resolver(
            StreamLayerMarker::new(),
            AlphaBaseMiddleware::new(),
            builder,
        )
    }
}
