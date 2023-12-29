use std::marker::PhantomData;

use crate::{
    error::{private::IntoResolverError, ExecError},
    internal::{
        build::build,
        middleware::{MiddlewareBuilder, MiddlewareLayerBuilder},
        resolver::{QueryOrMutation, Subscription},
    },
    layer::{DynLayer, Layer},
    middleware_from_core::{ProcedureKind, RequestContext},
    procedure_store::ProcedureTodo,
    router, ProcedureBuildFn, ProcedureDef,
};

use super::{
    middleware::Middleware,
    resolver::{IntoResolverResponse, LayerFn},
};

use futures::stream;
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
        // Given you can't attach middleware after the resolver (and supporting that would be painful)
        // we just type-erased everything as much as possible so it's less work on the compiler.

        let layer = LayerFn::new(|ctx: TMiddleware::LayerCtx, input, req| {
            // TODO: Make this work

            // let stream = (resolver)(
            //     ctx,
            //     serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
            // )
            // .to_stream();

            // Ok(stream.map(|v| match v {
            //     Ok(v) => serde_json::to_value(v).map_err(ExecError::SerializingResultErr),
            //     Err(e) => Err(ExecError::Resolver(e.into_resolver_error())),
            // }))

            Ok(stream::iter([Ok::<Value, ExecError>(Value::Null); 0]))
        });

        // In debug mode we box both the function and the stream.
        // This logic is that it will reduce monomorphisation and improve debug builds.
        // TODO: This needs more benchmarking. Should we always box the `Fn`??? Does boxing the `Stream` actually help build performance????
        #[cfg(debug_assertions)]
        let layer = layer.erased();

        let dyn_layer = boxed(self.0.mw.build(layer));

        let build: ProcedureBuildFn<TMiddleware::Ctx> = Box::new(move |key, ctx| {
            // TODO: correct `ProcedureKind`
            // build(key, ctx, ProcedureKind::Query, self.0.mw.build(resolver))

            ctx.queries.insert(
                key.to_string(),
                ProcedureTodo {
                    exec: dyn_layer,
                    // TODO: Correct types
                    ty: ProcedureDef {
                        key: "todo".into(),
                        input: specta::DataType::Any,
                        result: specta::DataType::Any,
                        error: specta::DataType::Any,
                    },
                },
            );
        });

        Procedure(HasResolver { build })
    }
}

impl<TCtx> Procedure<HasResolver<TCtx>> {
    pub(crate) fn take(self) -> ProcedureBuildFn<TCtx> {
        self.0.build
    }
}

pub(crate) fn boxed<TLCtx: Send + 'static>(layer: impl Layer<TLCtx>) -> Box<dyn DynLayer<TLCtx>> {
    Box::new(layer)
}
