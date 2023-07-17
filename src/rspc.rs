use std::marker::PhantomData;

use crate::{
    internal::{
        middleware::{
            BaseMiddleware, ConstrainedMiddleware, MiddlewareLayerBuilder, ProcedureKind,
        },
        procedure::{MissingResolver, Procedure},
        FutureMarkerType, RequestLayer, ResolverFunction, SealedRequestLayer, StreamMarkerType,
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
        Router::_internal_new()
    }

    pub fn with<Mw: ConstrainedMiddleware<TCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<Mw::NewCtx>, MiddlewareLayerBuilder<BaseMiddleware<TCtx>, Mw>>
    {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                middleware: BaseMiddleware::default(),
                mw,
            },
        )
    }

    pub fn query<R, RMarker>(self, resolver: R) -> Procedure<RMarker, BaseMiddleware<TCtx>>
    where
        R: ResolverFunction<TCtx, RMarker>,
        R::Result: RequestLayer<R::RequestMarker, Type = FutureMarkerType>,
    {
        Procedure::new(
            resolver.into_marker(ProcedureKind::Query),
            BaseMiddleware::default(),
        )
    }

    pub fn mutation<R, RMarker>(self, resolver: R) -> Procedure<RMarker, BaseMiddleware<TCtx>>
    where
        R: ResolverFunction<TCtx, RMarker>,
        R::Result: RequestLayer<R::RequestMarker, Type = FutureMarkerType>,
    {
        Procedure::new(
            resolver.into_marker(ProcedureKind::Mutation),
            BaseMiddleware::default(),
        )
    }

    pub fn subscription<R, RMarker>(self, resolver: R) -> Procedure<RMarker, BaseMiddleware<TCtx>>
    where
        R: ResolverFunction<TCtx, RMarker>,
        R::Result: RequestLayer<R::RequestMarker, Type = StreamMarkerType>,
    {
        Procedure::new(
            resolver.into_marker(ProcedureKind::Subscription),
            BaseMiddleware::default(),
        )
    }
}
