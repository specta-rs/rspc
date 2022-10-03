use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, RwLock},
};

use futures::Stream;
use serde::{de::DeserializeOwned, Serialize};
use specta::{Type, TypeDefs};

use crate::{
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, MiddlewareBuilderLike, MiddlewareLayerBuilder,
        MiddlewareMerger, ProcedureStore, ResolverLayer, UnbuiltProcedureBuilder,
    },
    Config, DoubleArgStreamMarker, ExecError, MiddlewareBuilder, MiddlewareLike, RequestResolver,
    RequestResult, Router, StreamResolver,
};

pub type GlobalData = Arc<RwLock<HashMap<TypeId, Box<dyn Any>>>>;

pub struct RouterBuilder<
    TCtx = (), // The is the context the current router was initialised with
    TMeta = (),
    TMiddleware = BaseMiddleware<TCtx>,
> where
    TCtx: Send + Sync + 'static,
    TMeta: Send + 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx> + Send + 'static,
{
    data: GlobalData,
    config: Config,
    middleware: TMiddleware,
    queries: ProcedureStore<TCtx>,
    mutations: ProcedureStore<TCtx>,
    subscriptions: ProcedureStore<TCtx>,
    typ_store: TypeDefs,
    phantom: PhantomData<TMeta>,
}

#[allow(clippy::new_without_default, clippy::new_ret_no_self)]
impl<TCtx, TMeta> Router<TCtx, TMeta>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + 'static,
{
    pub fn new() -> RouterBuilder<TCtx, TMeta, BaseMiddleware<TCtx>> {
        RouterBuilder::new()
    }
}

#[allow(clippy::new_without_default)]
impl<TCtx, TMeta> RouterBuilder<TCtx, TMeta, BaseMiddleware<TCtx>>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            data: GlobalData::default(),
            config: Config::new(),
            middleware: BaseMiddleware::default(),
            queries: ProcedureStore::new("query"),
            mutations: ProcedureStore::new("mutation"),
            subscriptions: ProcedureStore::new("subscription"),
            typ_store: TypeDefs::new(),
            phantom: PhantomData,
        }
    }
}

impl<TCtx, TLayerCtx, TMeta, TMiddleware> RouterBuilder<TCtx, TMeta, TMiddleware>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + 'static,
    TLayerCtx: Send + Sync + 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx, LayerContext = TLayerCtx> + Send + 'static,
{
    /// Attach a configuration to the router. Calling this multiple times will overwrite the previous config.
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn middleware<TNewMiddleware, TNewLayerCtx>(
        self,
        builder: impl Fn(MiddlewareBuilder<TLayerCtx>) -> TNewMiddleware,
    ) -> RouterBuilder<
        TCtx,
        TMeta,
        MiddlewareLayerBuilder<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TNewMiddleware>,
    >
    where
        TNewLayerCtx: Send + Sync + 'static,
        TNewMiddleware: MiddlewareLike<TLayerCtx, NewCtx = TNewLayerCtx> + Send + Sync + 'static,
    {
        let Self {
            data,
            config,
            middleware,
            queries,
            mutations,
            subscriptions,
            typ_store,
            ..
        } = self;

        let mw = builder(MiddlewareBuilder(PhantomData));
        RouterBuilder {
            data,
            config,
            middleware: MiddlewareLayerBuilder {
                middleware,
                mw,
                phantom: PhantomData,
            },
            queries,
            mutations,
            subscriptions,
            typ_store,
            phantom: PhantomData,
        }
    }

    pub fn query<
        TUnbuiltResolver,
        TUnbuiltResult,
        TUnbuiltResultMarker,
        TBuiltResolver,
        TBuiltResolverMarker,
        TBuiltResultMarker,
    >(
        mut self,
        key: &'static str,
        builder: impl Fn(
            UnbuiltProcedureBuilder<TLayerCtx, TUnbuiltResolver>,
        ) -> BuiltProcedureBuilder<TBuiltResolver>,
    ) -> Self
    where
        TUnbuiltResolver: Fn(TLayerCtx, TBuiltResolver::Arg) -> TUnbuiltResult,
        TUnbuiltResult: RequestResult<TUnbuiltResultMarker>,
        TBuiltResolver: RequestResolver<TLayerCtx, TBuiltResultMarker, TBuiltResolverMarker>,
    {
        let resolver = builder(UnbuiltProcedureBuilder::new(self.data.clone())).resolver;

        self.queries.append(
            key.into(),
            self.middleware.build(ResolverLayer {
                func: move |ctx, input, _| {
                    resolver
                        .exec(
                            ctx,
                            serde_json::from_value(input)
                                .map_err(ExecError::DeserializingArgErr)?,
                        )
                        .map(|v| v.to_request_future().into())
                },
                phantom: PhantomData,
            }),
            TBuiltResolver::typedef(&mut self.typ_store),
        );
        self
    }

    pub fn mutation<
        TUnbuiltResolver,
        TUnbuiltResult,
        TUnbuiltResultMarker,
        TBuiltResolver,
        TBuiltResolverMarker,
        TBuiltResultMarker,
    >(
        mut self,
        key: &'static str,
        builder: impl Fn(
            UnbuiltProcedureBuilder<TLayerCtx, TUnbuiltResolver>,
        ) -> BuiltProcedureBuilder<TBuiltResolver>,
    ) -> Self
    where
        TUnbuiltResolver: Fn(TLayerCtx, TBuiltResolver::Arg) -> TUnbuiltResult,
        TUnbuiltResult: RequestResult<TUnbuiltResultMarker>,
        TBuiltResolver: RequestResolver<TLayerCtx, TBuiltResolverMarker, TBuiltResultMarker>,
    {
        let resolver = builder(UnbuiltProcedureBuilder::new(self.data.clone())).resolver;
        self.mutations.append(
            key.into(),
            self.middleware.build(ResolverLayer {
                func: move |ctx, input, _| {
                    resolver
                        .exec(
                            ctx,
                            serde_json::from_value(input)
                                .map_err(ExecError::DeserializingArgErr)?,
                        )
                        .map(|v| v.to_request_future().into())
                },
                phantom: PhantomData,
            }),
            TBuiltResolver::typedef(&mut self.typ_store),
        );
        self
    }

    pub fn subscription<TResolver, TArg, TStream, TResult, TResultMarker>(
        mut self,
        key: &'static str,
        builder: impl Fn(
            UnbuiltProcedureBuilder<TLayerCtx, TResolver>,
        ) -> BuiltProcedureBuilder<TResolver>,
    ) -> Self
    where
        TArg: DeserializeOwned + Type,
        TStream: Stream<Item = TResult> + Send + 'static,
        TResult: Serialize + Type,
        TResolver: Fn(TLayerCtx, TArg) -> TStream
            + StreamResolver<TLayerCtx, DoubleArgStreamMarker<TArg, TResultMarker, TStream>>
            + Send
            + Sync
            + 'static,
    {
        let resolver = builder(UnbuiltProcedureBuilder::new(self.data.clone())).resolver;
        self.subscriptions.append(
            key.into(),
            self.middleware.build(ResolverLayer {
                func: move |ctx, input, _| {
                    resolver
                        .exec(
                            ctx,
                            serde_json::from_value(input)
                                .map_err(ExecError::DeserializingArgErr)?,
                        )
                        .map(Into::into)
                },
                phantom: PhantomData,
            }),
            TResolver::typedef(&mut self.typ_store),
        );
        self
    }

    pub fn merge<TNewLayerCtx, TIncomingMiddleware>(
        self,
        prefix: &'static str,
        router: RouterBuilder<TLayerCtx, TMeta, TIncomingMiddleware>,
    ) -> RouterBuilder<
        TCtx,
        TMeta,
        MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>,
    >
    where
        TNewLayerCtx: 'static,
        TIncomingMiddleware:
            MiddlewareBuilderLike<TLayerCtx, LayerContext = TNewLayerCtx> + Send + 'static,
    {
        #[allow(clippy::panic)]
        if prefix.is_empty() || prefix.starts_with("rpc.") || prefix.starts_with("rspc.") {
            panic!(
                "rspc error: attempted to merge a router with the prefix '{}', however this name is not allowed.",
                prefix
            );
        }

        let Self {
            data,
            config,
            middleware,
            mut queries,
            mut mutations,
            mut subscriptions,
            mut typ_store,
            ..
        } = self;

        for (key, query) in router.queries.store {
            queries.append(
                format!("{}{}", prefix, key),
                middleware.build(query.exec),
                query.ty,
            );
        }

        for (key, mutation) in router.mutations.store {
            mutations.append(
                format!("{}{}", prefix, key),
                middleware.build(mutation.exec),
                mutation.ty,
            );
        }

        for (key, subscription) in router.subscriptions.store {
            subscriptions.append(
                format!("{}{}", prefix, key),
                middleware.build(subscription.exec),
                subscription.ty,
            );
        }

        for (name, typ) in router.typ_store {
            typ_store.insert(name, typ);
        }

        RouterBuilder {
            data,
            config,
            middleware: MiddlewareMerger {
                middleware,
                middleware2: router.middleware,
                phantom: PhantomData,
            },
            queries,
            mutations,
            subscriptions,
            typ_store,
            phantom: PhantomData,
        }
    }

    pub fn build(self) -> Router<TCtx, TMeta> {
        let Self {
            config,
            queries,
            mutations,
            subscriptions,
            typ_store,
            ..
        } = self;

        let export_path = config.export_bindings_on_build.clone();
        let router = Router {
            config,
            queries,
            mutations,
            subscriptions,
            typ_store,
            phantom: PhantomData,
        };

        #[cfg(debug_assertions)]
        #[allow(clippy::unwrap_used)]
        if let Some(export_path) = export_path {
            router.export_ts(export_path).unwrap();
        }

        router
    }
}
