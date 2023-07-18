use std::marker::PhantomData;

use crate::{
    internal::{
        middleware::{
            BaseMiddleware, ConstrainedMiddleware, Middleware, MiddlewareLayerBuilder,
            ProcedureKind,
        },
        procedure::{MissingResolver, Procedure},
        FutureMarkerType, RequestLayer, ResolverFunction, StreamMarkerType,
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

macro_rules! resolver {
    ($func:ident, $kind:ident, $result_marker:ident) => {
        pub fn $func<R, RMarker>(self, resolver: R) -> Procedure<RMarker, BaseMiddleware<TCtx>>
        where
            R: ResolverFunction<TCtx, RMarker>,
            R::Result: RequestLayer<R::RequestMarker, Type = $result_marker>,
        {
            Procedure::new(
                resolver.into_marker(ProcedureKind::$kind),
                BaseMiddleware::default(),
            )
        }
    };
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
    ) -> Procedure<MissingResolver, MiddlewareLayerBuilder<BaseMiddleware<TCtx>, Mw>> {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                middleware: BaseMiddleware::default(),
                mw,
            },
        )
    }

    #[cfg(feature = "unstable")]
    pub fn with2<Mw: Middleware<TCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver, MiddlewareLayerBuilder<BaseMiddleware<TCtx>, Mw>> {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                middleware: BaseMiddleware::default(),
                mw,
            },
        )
    }

    resolver!(query, Query, FutureMarkerType);
    resolver!(mutation, Mutation, FutureMarkerType);
    resolver!(subscription, Subscription, StreamMarkerType);
}
