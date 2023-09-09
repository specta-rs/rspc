use std::{borrow::Cow, future::ready, marker::PhantomData};

use futures::stream;

use crate::internal::{
    middleware::{ConstrainedMiddleware, MiddlewareBuilder, MiddlewareLayerBuilder, ProcedureKind},
    procedure::BuildProceduresCtx,
    resolver::{HasResolver, ResolverFunction, ResolverLayer, StreamAdapter},
};

/// TODO: Explain
pub struct MissingResolver<TError>(PhantomData<TError>);

impl<TError> Default for MissingResolver<TError> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// TODO
pub struct Procedure<T, TMiddleware> {
    pub(crate) resolver: T,
    pub(crate) mw: TMiddleware,
}

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
        // TODO: Bring this back
        // pub fn $func<R, M>(self, resolver: R) -> Procedure<HasResolver<R, TResult, M>, TMiddleware>
        // where
        //     HasResolver<R, TResultM>: ResolverFunction<TMiddleware::LayerCtx, TError>,
        // {
        //     Procedure::new(HasResolver::new(resolver, ProcedureKind::$kind), self.mw)
        // }
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

impl<F, M, TResult, TMiddleware> Procedure<HasResolver<F, TResult, M>, TMiddleware> {
    pub(crate) fn build(
        self,
        key: Cow<'static, str>,
        ctx: &mut BuildProceduresCtx<'_, TMiddleware::Ctx>,
    )
    // TODO: Applying these sorta bounds here is cursed but it helps for refactoring
    where
        HasResolver<F, TResult, M>: ResolverFunction<TMiddleware::LayerCtx>,
        TMiddleware: MiddlewareBuilder,
    {
        let key_str = key.to_string();
        let type_def = self
            .resolver
            .into_procedure_def(key, &mut ctx.ty_store)
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
