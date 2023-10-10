use std::{borrow::Cow, marker::PhantomData};

use serde::de::DeserializeOwned;
use specta::Type;

use crate::internal::{
    middleware::{MiddlewareBuilder, MiddlewareFn, MiddlewareLayerBuilder},
    resolver::{
        FutureMarkerType, HasResolver, RequestLayer, ResolverFunction, ResolverLayer,
        StreamMarkerType,
    },
};
use rspc_core::internal::{router::Router, ProcedureKind};

/// TODO: Explain
pub struct MissingResolver<TError>(PhantomData<TError>);

impl<TError> Default for MissingResolver<TError> {
    fn default() -> Self {
        Self(Default::default())
    }
}

mod private {
    pub struct Procedure<T, TMiddleware> {
        pub(crate) resolver: T,
        pub(crate) mw: TMiddleware,
    }
}

pub(crate) use private::Procedure;

impl<T, TMiddleware> Procedure<T, TMiddleware>
where
    TMiddleware: MiddlewareBuilder,
{
    pub(crate) fn new(resolver: T, mw: TMiddleware) -> Self {
        Self { resolver, mw }
    }
}

macro_rules! resolver {
    ($func:ident, $kind:ident, $result_marker:ident) => {
        pub fn $func<R, RMarker>(self, resolver: R) -> Procedure<RMarker, TMiddleware>
        where
            R: ResolverFunction<TMiddleware::LayerCtx, RMarker>,
            R::Result: RequestLayer<R::RequestMarker, TypeMarker = $result_marker, Error = TError>,
        {
            Procedure::new(resolver.into_marker(ProcedureKind::$kind), self.mw)
        }
    };
}

// Can only set the resolver or add middleware until a resolver has been set.
// Eg. `.query().subscription()` makes no sense.
impl<TMiddleware, TError> Procedure<MissingResolver<TError>, TMiddleware>
where
    TMiddleware: MiddlewareBuilder,
{
    resolver!(query, Query, FutureMarkerType);
    resolver!(mutation, Mutation, FutureMarkerType);
    resolver!(subscription, Subscription, StreamMarkerType);

    pub fn error(self) -> Procedure<MissingResolver<TError>, TMiddleware> {
        Procedure {
            resolver: self.resolver,
            mw: self.mw,
        }
    }

    pub fn with<TNewCtx, Mw>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<TError>, MiddlewareLayerBuilder<TMiddleware, Mw, TNewCtx>>
    where
        TNewCtx: Send + Sync + 'static,
        Mw: MiddlewareFn<TMiddleware::LayerCtx, TNewCtx>,
    {
        Procedure::new(
            MissingResolver::default(),
            MiddlewareLayerBuilder {
                // todo: enforce via typestate
                middleware: self.mw,
                mw,
                phantom: PhantomData,
            },
        )
    }
}

impl<F, TArg, TResult, TResultMarker, TMiddleware>
    Procedure<HasResolver<F, TMiddleware::LayerCtx, TArg, TResult, TResultMarker>, TMiddleware>
where
    F: Fn(TMiddleware::LayerCtx, TArg) -> TResult + Send + Sync + 'static,
    TArg: Type + DeserializeOwned + 'static,
    TResult: RequestLayer<TResultMarker> + 'static,
    TResultMarker: 'static,
    TMiddleware: MiddlewareBuilder,
{
    pub(crate) fn build(self, key: Cow<'static, str>, ctx: &mut Router<TMiddleware::Ctx>) {
        let HasResolver(resolver, kind, _) = self.resolver;

        rspc_core::internal::build::<
            TMiddleware::Ctx,
            TMiddleware::Arg<TArg>,
            TResult::Result,
            TResult::Error,
        >(
            key,
            ctx,
            kind,
            self.mw.build(ResolverLayer::new(move |ctx, input, _| {
                Ok((resolver)(ctx, input).exec())
            })),
        );
    }
}
