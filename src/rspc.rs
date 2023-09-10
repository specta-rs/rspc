use std::marker::PhantomData;

use crate::{
    internal::{
        middleware::{
            BaseMiddleware, ConstrainedMiddleware, MiddlewareLayerBuilder, ProcedureKind,
        },
        procedure::{MissingResolver, Procedure},
        resolver::HasResolver,
        Layer,
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

// // TODO: Rename
// pub struct Todo<TResult, TOk, TError, TMarker>(PhantomData<(TResult, TOk, TError, TMarker)>)
// where
//     TResult: IntoQueryMutationResponse<TMarker>,
//     TResult::Stream: Stream<Item = Result<TOk, TError>>;

macro_rules! resolver {
    // TODO: Subscriptions are different
    ($func:ident, $kind:ident) => {
        pub fn $func<R, M>(
            self,
            resolver: R,
        ) -> Procedure<HasResolver<R, TError, M>, BaseMiddleware<TCtx>>
        where
            // TODO: Subscription's won't work
            HasResolver<R, TError, M>: Layer<TCtx>,
        {
            let resolver = HasResolver::new(resolver, ProcedureKind::$kind);

            // TODO: Make this work
            // // Trade runtime performance for reduced monomorphization
            // #[cfg(debug_assertions)]
            // let resolver = boxed(resolver);

            Procedure::new(resolver, BaseMiddleware::default())
        }
    };
}

impl<TCtx, TError> Rspc<TCtx, TError>
where
    TCtx: Send + Sync + 'static,
    TError: IntoResolverError,
{
    pub fn router(&self) -> Router<TCtx, TError> {
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

    resolver!(query, Query);
    resolver!(mutation, Mutation);
    resolver!(subscription, Subscription);
}
