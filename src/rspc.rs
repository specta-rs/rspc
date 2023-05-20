use std::marker::PhantomData;

use crate::{
    internal::{
        middleware::{
            BaseMiddleware, ConstrainedMiddleware, MiddlewareLayerBuilder, MissingResolver,
        },
        procedure::Procedure,
        FutureMarkerType, RequestKind, RequestLayer, RequestLayerMarker, ResolverFunction,
        SealedRequestLayer, StreamLayerMarker, StreamMarkerType,
    },
    Router,
};

/// Rspc is a starting point for constructing rspc procedures or routers.
///
/// This method supports const contexts so it can be instantiated at the top level as reuse across the whole application.
///
/// ```rust
/// use rspc::Rspc;
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
    pub fn router(&self) -> Router<TCtx> {
        Router::new()
    }

    pub fn with<Mw: ConstrainedMiddleware<TCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<Mw::NewCtx>, MiddlewareLayerBuilder<BaseMiddleware<TCtx>, Mw>, ()>
    {
        Procedure::new_from_middleware(MiddlewareLayerBuilder {
            middleware: BaseMiddleware::new(),
            mw,
        })
    }

    #[cfg(feature = "unstable")]
    pub fn with2<Mw: crate::internal::middleware::Middleware<TCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<Mw::NewCtx>, MiddlewareLayerBuilder<BaseMiddleware<TCtx>, Mw>, ()>
    {
        Procedure::new_from_middleware(MiddlewareLayerBuilder {
            middleware: BaseMiddleware::new(),
            mw,
        })
    }

    pub fn query<R, RMarker>(
        self,
        resolver: R,
    ) -> Procedure<R, BaseMiddleware<TCtx>, RequestLayerMarker<RMarker>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>
            + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
    {
        Procedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Query),
            BaseMiddleware::new(),
            resolver,
        )
    }

    pub fn mutation<R, RMarker>(
        self,
        resolver: R,
    ) -> Procedure<R, BaseMiddleware<TCtx>, RequestLayerMarker<RMarker>>
    where
        R: ResolverFunction<RequestLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>
            + SealedRequestLayer<R::RequestMarker, Type = FutureMarkerType>,
    {
        Procedure::new_from_resolver(
            RequestLayerMarker::new(RequestKind::Mutation),
            BaseMiddleware::new(),
            resolver,
        )
    }

    pub fn subscription<R, RMarker>(
        self,
        resolver: R,
    ) -> Procedure<R, BaseMiddleware<TCtx>, StreamLayerMarker<RMarker>>
    where
        R: ResolverFunction<StreamLayerMarker<RMarker>, LayerCtx = TCtx>
            + Fn(TCtx, R::Arg) -> R::Result,
        R::Result: RequestLayer<R::RequestMarker>
            + SealedRequestLayer<R::RequestMarker, Type = StreamMarkerType>,
    {
        Procedure::new_from_resolver(StreamLayerMarker::new(), BaseMiddleware::new(), resolver)
    }
}
