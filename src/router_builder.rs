use std::marker::PhantomData;

use futures::Stream;
use serde::{de::DeserializeOwned, Serialize};
use specta::{Type, TypeDefs};

use crate::{
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, MiddlewareBuilderLike, MiddlewareLayerBuilder,
        MiddlewareMerger, ProcedureStore, ResolverLayer, UnbuiltProcedureBuilder,
    },
    internal::{
        DoubleArgStreamMarker, GlobalData, MiddlewareBuilder, MiddlewareLike, ProcedureKind,
        RequestResolver, RequestResult, StreamResolver,
    },
    Config, ExecError, Router,
};

pub(crate) fn is_valid_procedure_name(s: &str) -> bool {
    s.is_empty()
        || s == "ws"
        || s.starts_with("rpc.")
        || s.starts_with("rspc.")
        || !s
            .chars()
            .all(|c| c.is_alphabetic() || c.is_numeric() || c == '.' || c == '_')
}

pub struct RouterBuilder<
    TCtx = (), // The is the context the current router was initialised with
    TMiddleware = BaseMiddleware<TCtx>,
> where
    TCtx: Send + Sync + 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx> + Send + 'static,
{
    data: GlobalData,
    config: Config,
    middleware: TMiddleware,
    queries: ProcedureStore<TCtx>,
    mutations: ProcedureStore<TCtx>,
    subscriptions: ProcedureStore<TCtx>,
    typ_store: TypeDefs,
}

pub trait RouterBuilderLike<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    type Middleware: MiddlewareBuilderLike<TCtx> + Send + 'static;

    fn expose(self) -> RouterBuilder<TCtx, Self::Middleware>;
}

impl<TCtx, TMiddleware> RouterBuilderLike<TCtx> for RouterBuilder<TCtx, TMiddleware>
where
    TCtx: Send + Sync + 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx> + Send + 'static,
{
    type Middleware = TMiddleware;

    fn expose(self) -> RouterBuilder<TCtx, Self::Middleware> {
        self
    }
}

#[allow(clippy::new_without_default, clippy::new_ret_no_self)]
impl<TCtx> Router<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    pub fn new() -> RouterBuilder<TCtx, BaseMiddleware<TCtx>> {
        RouterBuilder::new()
    }
}

#[allow(clippy::new_without_default)]
impl<TCtx> RouterBuilder<TCtx, BaseMiddleware<TCtx>>
where
    TCtx: Send + Sync + 'static,
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
        }
    }
}

impl<TCtx, TLayerCtx, TMiddleware> RouterBuilder<TCtx, TMiddleware>
where
    TCtx: Send + Sync + 'static,
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
        builder: impl FnOnce(
            UnbuiltProcedureBuilder<TLayerCtx, TUnbuiltResolver>,
        ) -> BuiltProcedureBuilder<TBuiltResolver>,
    ) -> Self
    where
        TUnbuiltResolver:
            Fn(TLayerCtx, TBuiltResolver::Arg) -> TUnbuiltResult + Send + Sync + 'static,
        TUnbuiltResult: RequestResult<TUnbuiltResultMarker>,
        TBuiltResolver: RequestResolver<TLayerCtx, TBuiltResultMarker, TBuiltResolverMarker>,
    {
        let built_procedure = builder(UnbuiltProcedureBuilder::new(
            key,
            ProcedureKind::Query,
            TUnbuiltResolver::typedef(&mut self.typ_store, key).unwrap(), // TODO: Unwrap is bad
            self.data.clone(),
        ));
        let resolver = built_procedure.resolver;

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
            built_procedure.typedef,
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
        builder: impl FnOnce(
            UnbuiltProcedureBuilder<TLayerCtx, TUnbuiltResolver>,
        ) -> BuiltProcedureBuilder<TBuiltResolver>,
    ) -> Self
    where
        TUnbuiltResolver:
            Fn(TLayerCtx, TBuiltResolver::Arg) -> TUnbuiltResult + Send + Sync + 'static,
        TUnbuiltResult: RequestResult<TUnbuiltResultMarker>,
        TBuiltResolver: RequestResolver<TLayerCtx, TBuiltResolverMarker, TBuiltResultMarker>,
    {
        let built_procedure = builder(UnbuiltProcedureBuilder::new(
            key,
            ProcedureKind::Mutation,
            TUnbuiltResolver::typedef(&mut self.typ_store, key).unwrap(), // TODO: Unwrap is bad
            self.data.clone(),
        ));
        let resolver = built_procedure.resolver;
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
            built_procedure.typedef,
        );
        self
    }

    pub fn subscription<TResolver, TArg, TStream, TResult, TResultMarker>(
        mut self,
        key: &'static str,
        builder: impl FnOnce(
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
        let built_procedure = builder(UnbuiltProcedureBuilder::new(
            key,
            ProcedureKind::Subscription,
            TResolver::typedef(&mut self.typ_store, key).unwrap(), // TODO: Unwrap is bad
            self.data.clone(),
        ));
        let resolver = built_procedure.resolver;
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
            built_procedure.typedef,
        );
        self
    }

    pub fn merge<TNewLayerCtx, TIncomingMiddleware>(
        self,
        prefix: &'static str,
        router: impl RouterBuilderLike<TLayerCtx, Middleware = TIncomingMiddleware>,
    ) -> RouterBuilder<
        TCtx,
        MiddlewareMerger<TCtx, TLayerCtx, TNewLayerCtx, TMiddleware, TIncomingMiddleware>,
    >
    where
        TNewLayerCtx: Send + Sync + 'static,
        TIncomingMiddleware:
            MiddlewareBuilderLike<TLayerCtx, LayerContext = TNewLayerCtx> + Send + 'static,
    {
        let router = router.expose();

        #[allow(clippy::panic)]
        if is_valid_procedure_name(prefix) {
            panic!(
                "rspc error: attempted to merge a router with the prefix '{}', however this name is not allowed.",
                prefix
            );
        }

        // TODO: The `data` field has gotta flow from the root router to the leaf routers so that we don't have to merge user defined types.

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

        for (key, mut query) in router.queries.store {
            query.ty.key = format!("{}{}", prefix, key);
            queries.append(
                format!("{}{}", prefix, key),
                middleware.build(query.exec),
                query.ty,
            );
        }

        for (key, mut mutation) in router.mutations.store {
            mutation.ty.key = format!("{}{}", prefix, key);
            mutations.append(
                format!("{}{}", prefix, key),
                middleware.build(mutation.exec),
                mutation.ty,
            );
        }

        for (key, mut subscription) in router.subscriptions.store {
            subscription.ty.key = format!("{}{}", prefix, key);
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
        }
    }

    // TODO: Remove this
    // It allows for merging a router without the `TMiddleware` and `TCtx` "flowing" out of the child router.
    // This is required to mount more than two routers which use `impl RouterBuilderLike`.
    // This should be fixed with the new syntax.
    pub fn yolo_merge(
        mut self,
        prefix: &'static str,
        router: impl RouterBuilderLike<TLayerCtx>,
    ) -> Self {
        let router = router.expose();

        #[allow(clippy::panic)]
        if is_valid_procedure_name(prefix) {
            panic!(
                "rspc error: attempted to merge a router with the prefix '{}', however this name is not allowed.",
                prefix
            );
        }

        // TODO: The `data` field has gotta flow from the root router to the leaf routers so that we don't have to merge user defined types.

        for (key, mut query) in router.queries.store {
            query.ty.key = format!("{}{}", prefix, key);
            self.queries.append(
                format!("{}{}", prefix, key),
                self.middleware.build(query.exec),
                query.ty,
            );
        }

        for (key, mut mutation) in router.mutations.store {
            mutation.ty.key = format!("{}{}", prefix, key);
            self.mutations.append(
                format!("{}{}", prefix, key),
                self.middleware.build(mutation.exec),
                mutation.ty,
            );
        }

        for (key, mut subscription) in router.subscriptions.store {
            subscription.ty.key = format!("{}{}", prefix, key);
            self.subscriptions.append(
                format!("{}{}", prefix, key),
                self.middleware.build(subscription.exec),
                subscription.ty,
            );
        }

        for (name, typ) in router.typ_store {
            self.typ_store.insert(name, typ);
        }

        self
    }

    pub fn build(self) -> Router<TCtx> {
        let Self {
            data,
            config,
            queries,
            mutations,
            subscriptions,
            typ_store,
            ..
        } = self;

        let export_path = config.export_bindings_on_build.clone();
        let router = Router {
            data,
            config,
            queries,
            mutations,
            subscriptions,
            typ_store,
        };

        #[cfg(debug_assertions)]
        #[allow(clippy::unwrap_used)]
        if let Some(export_path) = export_path {
            router.export_ts(export_path).unwrap();
        }

        router
    }
}
