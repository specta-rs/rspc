use std::marker::PhantomData;

use crate::{
    internal::{
        middleware::{
            BaseMiddleware, ConstrainedMiddleware, MiddlewareLayerBuilder, ProcedureKind,
        },
        procedure::{MissingResolver, Procedure},
        resolver::{FutureMarkerType, RequestLayer, ResolverFunction, StreamMarkerType},
    },
    Infallible, IntoResolverError, Router,
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
    TCtx = (), // The is the context the current router was initialised with,
    TError = Infallible,
> where
    TCtx: Send + Sync + 'static,
    TError: IntoResolverError,
{
    phantom: PhantomData<(TCtx, TError)>,
}

#[allow(clippy::new_without_default)]
impl<TCtx, TError> Rspc<TCtx, TError>
where
    TCtx: Send + Sync + 'static,
    TError: IntoResolverError,
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
            R::Result: RequestLayer<R::RequestMarker, TypeMarker = $result_marker, Error = TError>,
        {
            Procedure::new(
                resolver.into_marker(ProcedureKind::$kind),
                BaseMiddleware::default(),
            )
        }
    };
}

impl<TCtx, TError> Rspc<TCtx, TError>
where
    TCtx: Send + Sync + 'static,
    TError: IntoResolverError,
{
    pub fn router(&self) -> Router<TCtx> {
        Router::_internal_new()
    }

    pub fn error<TNewError>(self) -> Procedure<MissingResolver<TNewError>, BaseMiddleware<TCtx>> {
        Procedure::new(MissingResolver::default(), BaseMiddleware::default())
    }

    pub fn with<Mw: ConstrainedMiddleware<TCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<Infallible>, MiddlewareLayerBuilder<BaseMiddleware<TCtx>, Mw>>
    {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                middleware: BaseMiddleware::default(),
                mw,
            },
        )
    }

    #[cfg(feature = "unstable")]
    pub fn with2<Mw: crate::internal::middleware::Middleware<TCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<Infallible>, MiddlewareLayerBuilder<BaseMiddleware<TCtx>, Mw>>
    {
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
