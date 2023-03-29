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
    middleware::AlphaMiddlewareContext, procedure::AlphaProcedure, AlphaBaseMiddleware,
    AlphaMiddlewareBuilder, AlphaMiddlewareBuilderLike, AlphaMiddlewareLayerBuilder,
    AlphaMiddlewareLike, AlphaRouter, MiddlewareArgMapper, MissingResolver, MwV2, MwV2Result,
    RequestKind, RequestLayerMarker, ResolverFunction, StreamLayerMarker,
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

    pub fn with<TMarker, Mw>(
        self,
        mw: Mw,
    ) -> AlphaProcedure<
        MissingResolver<Mw::NewCtx>,
        (),
        AlphaMiddlewareLayerBuilder<AlphaBaseMiddleware<TCtx>, Mw, TMarker>,
    >
    where
        TMarker: Send + Sync + 'static,
        Mw: MwV2<TCtx, TMarker>
            + Fn(
                AlphaMiddlewareContext<
                    <<Mw::Result as MwV2Result>::MwMapper as MiddlewareArgMapper>::State,
                >,
                TCtx,
            ) -> Mw::Fut
            + Send
            + Sync
            + 'static,
    {
        // let mw = builder(AlphaMiddlewareBuilder(PhantomData));
        // AlphaProcedure::new_from_middleware(AlphaMiddlewareLayerBuilder {
        //     middleware: AlphaBaseMiddleware::new(),
        //     mw,
        // })

        todo!();
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
