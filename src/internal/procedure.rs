use std::marker::PhantomData;

use crate::{
    error::private::IntoResolverError,
    internal::{
        build::build,
        middleware::{MiddlewareBuilder, MiddlewareLayerBuilder},
        resolver::{QueryOrMutation, Subscription},
    },
    layer::Layer,
    middleware_from_core::{ProcedureKind, RequestContext},
    ProcedureBuildFn,
};

use super::{middleware::Middleware, resolver::IntoResolverResponse};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use specta::Type;

/// TODO
pub struct Procedure<T>(T);

// These `MissingResolver` and `HasResolver` exist to make the typestate-pattern work
// We also erase all generics when changing state to reduce work for the compiler.

pub struct MissingResolver<TError, TMiddleware> {
    mw: TMiddleware,
    phantom: PhantomData<(TError, TMiddleware)>,
}

impl<TError, TMiddleware> MissingResolver<TError, TMiddleware> {
    pub fn new(mw: TMiddleware) -> Procedure<Self> {
        Procedure(Self {
            mw,
            phantom: PhantomData,
        })
    }
}

pub struct HasResolver<TCtx> {
    build: ProcedureBuildFn<TCtx>,
}

// Can only add middleware until the resolver and you can only set the resolver once.
// Eg. `.query().subscription()` makes no sense and `.query().with()` is going to be stupidly hard to maintain without breaking rspc's performance characteristics.
impl<TMiddleware, TError> Procedure<MissingResolver<TError, TMiddleware>>
where
    TMiddleware: MiddlewareBuilder,
    TError: IntoResolverError,
{
    pub fn error<TErr>(self) -> Procedure<MissingResolver<TErr, TMiddleware>> {
        MissingResolver::new(self.0.mw)
    }

    pub fn with<Mw: Middleware<TMiddleware::LayerCtx>>(
        self,
        mw: Mw,
    ) -> Procedure<MissingResolver<TError, MiddlewareLayerBuilder<TMiddleware, Mw>>> {
        MissingResolver::new(MiddlewareLayerBuilder {
            middleware: self.0.mw,
            mw,
        })
    }

    // resolvers!(this, TMiddleware::LayerCtx, TMiddleware, this.mw); // TODO: Bring back the rest of them

    pub fn query<F, TResult, TResultMarker, TArg>(
        self,
        resolver: F,
    ) -> Procedure<HasResolver<TMiddleware::Ctx>>
    where
        // TODO: Breaking these of into a struct??? Does that make the errors worse???
        F: Fn(TMiddleware::LayerCtx, TArg) -> TResult + Send + Sync + 'static,
        TArg: DeserializeOwned + Type + 'static,
        TResult: IntoResolverResponse<'static, TResultMarker, Err = TError>,
        TResult::Ok: Serialize + Type + 'static,
        TResultMarker: 'static,
    {
        // TODO: Erase the hell outta the whole chain here. You can't attach middleware after the resolver (and supporting that would be painful)

        // let y = |ctx: TMiddleware::LayerCtx, input: Value, _req: RequestContext| {};

        // let stream = (self.resolver)(
        //     ctx,
        //     serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
        // )
        // .to_stream();

        // Ok(stream.map(|v| match v {
        //     Ok(v) => serde_json::to_value(v).map_err(ExecError::SerializingResultErr),
        //     Err(e) => Err(ExecError::Resolver(e.into_resolver_error())),
        // }))

        // TODO: Make this work
        // // Trade runtime performance for reduced monomorphization
        // #[cfg(debug_assertions)]
        // let resolver = boxed(resolver);

        let build: ProcedureBuildFn<TMiddleware::Ctx> = Box::new(move |key, ctx| {
            // TODO: correct `ProcedureKind`
            // build(key, ctx, ProcedureKind::Query, self.0.mw.build(resolver))
        });

        Procedure(HasResolver { build })
    }
}

impl<TCtx> Procedure<HasResolver<TCtx>> {
    pub(crate) fn take(self) -> ProcedureBuildFn<TCtx> {
        self.0.build
    }
}
