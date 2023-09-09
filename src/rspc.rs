use std::marker::PhantomData;

use futures::Stream;

use serde::Serialize;
use specta::Type;

use crate::{
    internal::{
        middleware::{
            BaseMiddleware, ConstrainedMiddleware, MiddlewareLayerBuilder, ProcedureKind,
        },
        procedure::{MissingResolver, Procedure},
        resolver::{HasResolver, IntoQueryMutationResponse, QueryMutationFn, ResolverFunction},
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

// TODO: Subscriptions are different
// TResult: IntoQueryMutationResponse<TMarker, TErr>,
// TResult::Ok: Serialize + Type + 'static,
// TResult::Err: IntoResolverError + 'static,
// TResult::Stream: Stream<Item = Result<TResult, TError>>,
// TODO: Move these bounds onto the generic def on impl block
// TError: 'static,

macro_rules! resolver {
    ($func:ident, $kind:ident) => {
        // TODO: Only a single marker?
        pub fn $func<R, TResult, M, TMarker>(
            self,
            resolver: R,
        ) -> Procedure<HasResolver<R, TResult, M>, BaseMiddleware<TCtx>>
        where
            HasResolver<R, TResult, M>: ResolverFunction<TCtx> + QueryMutationFn<TError, TMarker>,
        {
            // let resolver = Box::new(resolver);

            // TODO: Get type_def somehow
            // let ty: fn() = || {
            //     todo!();
            // };

            let resolver = HasResolver::new(resolver, ProcedureKind::$kind);

            // TODO: Cfg debug
            // let resolver: Box<dyn ResolverFunction<TCtx, TError>> = Box::new(resolver);

            Procedure::new(resolver, BaseMiddleware::default())
        }

        // pub fn $func<R, M>(self, resolver: R) -> Procedure<HasResolver<R, M>, BaseMiddleware<TCtx>>
        // where
        //     HasResolver<R, M>: ResolverFunction<TCtx, TError>,
        // {
        //     let resolver = HasResolver::new(resolver, ProcedureKind::$kind);

        //     // let resolver: Box<dyn ResolverFunction<TCtx, TError>> =
        //     //     Box::new(HasResolver::new(resolver, ProcedureKind::$kind));

        //     Procedure::new(resolver, BaseMiddleware::default())
        // }
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

    resolver!(query, Query);
    resolver!(mutation, Mutation);
    resolver!(subscription, Subscription);
}
