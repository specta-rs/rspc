use std::{borrow::Cow, marker::PhantomData};

use crate::internal::{
    middleware::{ConstrainedMiddleware, MiddlewareBuilder, MiddlewareLayerBuilder},
    resolver::{HasResolver, QueryOrMutation, Subscription},
};
use rspc_core::internal::{router::Router, Layer, ProcedureKind};

/// TODO: Explain
pub struct MissingResolver<TError>(PhantomData<TError>);

impl<TError> Default for MissingResolver<TError> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// TODO
pub struct Procedure<TResolverState, TMiddleware> {
    pub(crate) resolver: TResolverState,
    pub(crate) mw: TMiddleware,
}

impl<TResolverState, TMiddleware> Procedure<TResolverState, TMiddleware>
where
    TMiddleware: MiddlewareBuilder,
{
    pub(crate) fn new(resolver: TResolverState, mw: TMiddleware) -> Self {
        Self { resolver, mw }
    }
}

macro_rules! resolvers {
    ($this:tt, $ctx:ty, $mw_ty:ty, $mw:expr) => {
        resolvers!(impl; $this, $ctx, $mw_ty, $mw, query, QueryOrMutation, Query);
        resolvers!(impl; $this, $ctx, $mw_ty, $mw, mutation, QueryOrMutation, Mutation);
        resolvers!(impl; $this, $ctx, $mw_ty, $mw, subscription, Subscription, Subscription);
    };
    (impl; $this:tt, $ctx:ty, $mw_ty:ty, $mw:expr, $fn_name:ident, $marker:ident, $kind:ident) => {
        pub fn $fn_name<TResolver, TResultMarker, TArgMarker>(
            self,
            resolver: TResolver,
        ) -> Procedure<
            HasResolver<TResolver, TError, $marker<TResultMarker>, TArgMarker>,
            $mw_ty,
        >
        where
            HasResolver<TResolver, TError, $marker<TResultMarker>, TArgMarker>: Layer<$ctx>,
        {
        	let $this = self;
            let resolver = HasResolver::new(resolver, ProcedureKind::$kind);

            // TODO: Make this work
            // // Trade runtime performance for reduced monomorphization
            // #[cfg(debug_assertions)]
            // let resolver = boxed(resolver);

            Procedure::new(resolver, $mw)
        }
    };
}

pub(crate) use resolvers;

// Can only set the resolver or add middleware until a resolver has been set.
// Eg. `.query().subscription()` makes no sense.
impl<TMiddleware, TError> Procedure<MissingResolver<TError>, TMiddleware>
where
    TMiddleware: MiddlewareBuilder,
{
    pub fn error(self) -> Procedure<MissingResolver<TError>, TMiddleware> {
        Procedure {
            resolver: self.resolver,
            mw: self.mw,
        }
    }

    pub fn with<Mw: ConstrainedMiddleware<TMiddleware::LayerCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<TError>, MiddlewareLayerBuilder<TMiddleware, Mw>> {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                // todo: enforce via typestate
                middleware: self.mw,
                mw,
            },
        )
    }

    #[cfg(feature = "unstable")]
    pub fn with2<Mw: crate::internal::middleware::Middleware<TMiddleware::LayerCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<TError>, MiddlewareLayerBuilder<TMiddleware, Mw>> {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                // todo: enforce via typestate
                middleware: self.mw,
                mw,
            },
        )
    }

    resolvers!(this, TMiddleware::LayerCtx, TMiddleware, this.mw);
}
