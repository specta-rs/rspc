use std::{borrow::Cow, future::ready, marker::PhantomData};

use futures::stream::{self, Once};
use serde::de::DeserializeOwned;
use serde_json::Value;
use specta::Type;

use crate::internal::{
    middleware::{ConstrainedMiddleware, MiddlewareBuilder, MiddlewareLayerBuilder, ProcedureKind},
    procedure::{BuildProceduresCtx, ProcedureDef},
    resolver::{
        HasResolver, RequestLayer, ResolverFunction, ResolverFunctionGood, ResolverLayer,
        StreamAdapter,
    },
};

/// TODO: Explain
pub struct MissingResolver<TError>(PhantomData<TError>);

impl<TError> Default for MissingResolver<TError> {
    fn default() -> Self {
        Self(Default::default())
    }
}

mod private {
    use super::*;

    // TODO: Would this be useful public?
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
    ($func:ident, $kind:ident) => {
        pub fn $func<R, RMarker>(self, resolver: R) -> Procedure<RMarker, TMiddleware>
        where
            R: ResolverFunction<TMiddleware::LayerCtx, TError, RMarker>,
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
    resolver!(query, Query);
    resolver!(mutation, Mutation);
    resolver!(subscription, Subscription);

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
}

// TArg, TResult, TResultMarker,
// HasResolver<F, TMiddleware::LayerCtx, TArg, TResult, TResultMarker>
// TResolver: Fn(TMiddleware::LayerCtx, TArg) -> TResult + Send + Sync + 'static,
// TArg: Type + DeserializeOwned + 'static,
// TResult: RequestLayer<TResultMarker> + 'static,
// TResultMarker: 'static,
impl<F, M, TMiddleware, TError>
    Procedure<HasResolver<F, TMiddleware::LayerCtx, TError, M>, TMiddleware>
where
    // This bound is *really* lately applied so it's error will be shocking
    HasResolver<F, TMiddleware::LayerCtx, TError, M>:
        ResolverFunctionGood<TMiddleware::LayerCtx, TError>,
    TMiddleware: MiddlewareBuilder,
    // TODO: Remove the following bounds?
    // TResolver: Fn(TMiddleware::LayerCtx, TArg) -> TResult + Send + Sync + 'static,
{
    pub(crate) fn build(
        self,
        key: Cow<'static, str>,
        ctx: &mut BuildProceduresCtx<'_, TMiddleware::Ctx>,
    ) {
        let key_str = key.to_string();
        let type_def = self
            .resolver
            .into_procedure_def::<TMiddleware>(key, &mut ctx.ty_store)
            .expect("error exporting types"); // TODO: Error handling using `#[track_caller]`

        let m = match &self.resolver.kind {
            ProcedureKind::Query => &mut ctx.queries,
            ProcedureKind::Mutation => &mut ctx.mutations,
            ProcedureKind::Subscription => &mut ctx.subscriptions,
        };

        m.append(
            key_str,
            // TODO: Take in `serde_json::Value for argument at this stage
            self.mw.build(ResolverLayer::new(move |ctx, value, req| {
                // TODO: Lol this is so bad
                Ok(StreamAdapter {
                    stream: stream::once(ready(Ok(self.resolver.exec(ctx, value, req)))),
                })
            })),
            type_def,
        );
    }
}
