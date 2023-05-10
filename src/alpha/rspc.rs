use std::marker::PhantomData;

use super::{
    middleware::AlphaMiddlewareContext, procedure::AlphaProcedure, AlphaBaseMiddleware,
    AlphaMiddlewareLayerBuilder, AlphaRequestLayer, AlphaRouter, FutureMarker, MissingResolver,
    MwV2, MwV2Result, RequestKind, RequestLayerMarker, ResolverFunction, StreamLayerMarker,
    StreamMarker,
};

/// Rspc is a starting point for constructing rspc procedures or routers.
///
/// This method supports const contexts so it can be instantiated at the top level as reuse across the whole application.
///
/// ```rust
/// use rspc::alpha::Rspc;
///
/// const R: Rspc<()> = Rspc::new();
/// ```
pub struct Rspc<
    TCtx = (), // The is the context the current router was initialised with
> where
    TCtx: Send + Sync + 'static,
{
    phantom: PhantomData<TCtx>,
}

#[allow(clippy::new_without_default)]
impl<TCtx> Rspc<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    pub const fn new() -> Self {
        Self {
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

    pub fn with<Mw: MwV2<TCtx>>(
        self,
        mw: Mw,
    ) -> AlphaProcedure<
        MissingResolver<Mw::NewCtx>,
        crate::Error,
        (),
        AlphaMiddlewareLayerBuilder<AlphaBaseMiddleware<TCtx>, Mw>,
    > {
        AlphaProcedure::new_from_middleware(AlphaMiddlewareLayerBuilder {
            middleware: AlphaBaseMiddleware::new(),
            mw,
        })
    }

    #[cfg(feature = "unstable")]
    pub fn with2<Mw: super::MwV3<TCtx>>(
        self,
        mw: Mw,
    ) -> AlphaProcedure<
        MissingResolver<Mw::NewCtx>,
        crate::Error,
        (),
        AlphaMiddlewareLayerBuilder<AlphaBaseMiddleware<TCtx>, Mw>,
    > {
        AlphaProcedure::new_from_middleware(AlphaMiddlewareLayerBuilder {
            middleware: AlphaBaseMiddleware::new(),
            mw,
        })
    }

    pub fn query<R, RMarker>(
        self,
        resolver: R,
    ) -> AlphaProcedure<R, crate::Error, RequestLayerMarker<RMarker>, AlphaBaseMiddleware<TCtx>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, crate::Error, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: AlphaRequestLayer<R::RequestMarker, crate::Error, Type = FutureMarker>,
    {
        AlphaProcedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Query),
            AlphaBaseMiddleware::new(),
            resolver,
        )
    }

    pub fn mutation<R, RMarker>(
        self,
        resolver: R,
    ) -> AlphaProcedure<R, crate::Error, RequestLayerMarker<RMarker>, AlphaBaseMiddleware<TCtx>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, crate::Error, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: AlphaRequestLayer<R::RequestMarker, crate::Error, Type = FutureMarker>,
    {
        AlphaProcedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Mutation),
            AlphaBaseMiddleware::new(),
            resolver,
        )
    }

    pub fn subscription<R, RMarker>(
        self,
        resolver: R,
    ) -> AlphaProcedure<R, crate::Error, StreamLayerMarker<RMarker>, AlphaBaseMiddleware<TCtx>>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, crate::Error, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: AlphaRequestLayer<R::RequestMarker, crate::Error, Type = StreamMarker>,
    {
        AlphaProcedure::new_from_resolver(
            StreamLayerMarker::new(),
            AlphaBaseMiddleware::new(),
            resolver,
        )
    }

    // TODO: Put this onto the other thing
    // TODO: Trait on `E` + add a bunch of error types into Specta
    pub fn with_error<E>(
        self,
    ) -> AlphaProcedure<MissingResolver<TCtx>, E, (), AlphaBaseMiddleware<TCtx>> {
        todo!();
    }
}
