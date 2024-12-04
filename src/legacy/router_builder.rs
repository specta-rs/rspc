use std::marker::PhantomData;

use futures::Stream;
use serde::{de::DeserializeOwned, Serialize};
use specta::Type;
use specta::TypeMap;

use crate::{
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, MiddlewareBuilderLike, MiddlewareLayerBuilder,
        MiddlewareMerger, ProcedureStore, ResolverLayer, UnbuiltProcedureBuilder,
    },
    Config, DoubleArgStreamMarker, ExecError, MiddlewareBuilder, MiddlewareLike, RequestLayer,
    Resolver, Router, StreamResolver,
};

pub struct RouterBuilder<
    TCtx = (), // The is the context the current router was initialised with
    TMeta = (),
    TMiddleware = BaseMiddleware<TCtx>,
> where
    TCtx: Send + Sync + 'static,
    TMeta: Send + 'static,
    TMiddleware: MiddlewareBuilderLike<TCtx> + Send + 'static,
{
    config: Config,
    middleware: TMiddleware,
    queries: ProcedureStore<TCtx>,
    mutations: ProcedureStore<TCtx>,
    subscriptions: ProcedureStore<TCtx>,
    type_map: TypeMap,
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
            config: Config::new(),
            middleware: BaseMiddleware::default(),
            queries: ProcedureStore::new("query"),
            mutations: ProcedureStore::new("mutation"),
            subscriptions: ProcedureStore::new("subscription"),
            type_map: Default::default(),
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
    // /// Attach a configuration to the router. Calling this multiple times will overwrite the previous config.
    // pub fn config(mut self, config: Config) -> Self {
    //     self.config = config;
    //     self
    // }

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
            config,
            middleware,
            queries,
            mutations,
            subscriptions,
            type_map,
            ..
        } = self;

        let mw = builder(MiddlewareBuilder(PhantomData));
        RouterBuilder {
            config,
            middleware: MiddlewareLayerBuilder {
                middleware,
                mw,
                phantom: PhantomData,
            },
            queries,
            mutations,
            subscriptions,
            type_map,
            phantom: PhantomData,
        }
    }

    pub fn query<TResolver, TArg, TResult, TResultMarker>(
        mut self,
        key: &'static str,
        builder: impl Fn(
            UnbuiltProcedureBuilder<TLayerCtx, TResolver>,
        ) -> BuiltProcedureBuilder<TResolver>,
    ) -> Self
    where
        TArg: DeserializeOwned + Type,
        TResult: RequestLayer<TResultMarker>,
        TResolver: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    {
        let resolver = builder(UnbuiltProcedureBuilder::default()).resolver;
        self.queries.append(
            key.into(),
            self.middleware.build(ResolverLayer {
                func: move |ctx, input, _| {
                    resolver.exec(
                        ctx,
                        serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
                    )
                },
                phantom: PhantomData,
            }),
            TResolver::typedef(&mut self.type_map),
        );
        self
    }

    pub fn mutation<TResolver, TArg, TResult, TResultMarker>(
        mut self,
        key: &'static str,
        builder: impl Fn(
            UnbuiltProcedureBuilder<TLayerCtx, TResolver>,
        ) -> BuiltProcedureBuilder<TResolver>,
    ) -> Self
    where
        TArg: DeserializeOwned + Type,
        TResult: RequestLayer<TResultMarker>,
        TResolver: Fn(TLayerCtx, TArg) -> TResult + Send + Sync + 'static,
    {
        let resolver = builder(UnbuiltProcedureBuilder::default()).resolver;
        self.mutations.append(
            key.into(),
            self.middleware.build(ResolverLayer {
                func: move |ctx, input, _| {
                    resolver.exec(
                        ctx,
                        serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
                    )
                },
                phantom: PhantomData,
            }),
            TResolver::typedef(&mut self.type_map),
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
        let resolver = builder(UnbuiltProcedureBuilder::default()).resolver;
        self.subscriptions.append(
            key.into(),
            self.middleware.build(ResolverLayer {
                func: move |ctx, input, _| {
                    resolver.exec(
                        ctx,
                        serde_json::from_value(input).map_err(ExecError::DeserializingArgErr)?,
                    )
                },
                phantom: PhantomData,
            }),
            TResolver::typedef(&mut self.type_map),
        );
        self
    }

    pub fn merge<TNewLayerCtx, TIncomingMiddleware>(
        mut self,
        prefix: &'static str,
        router: RouterBuilder<TLayerCtx, TMeta, TIncomingMiddleware>,
    ) -> Self
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

        // TODO: The `data` field has gotta flow from the root router to the leaf routers so that we don't have to merge user defined types.

        for (key, query) in router.queries.store {
            // query.ty.key = format!("{}{}", prefix, key);
            self.queries.append(
                format!("{}{}", prefix, key),
                self.middleware.build(query.exec),
                query.ty,
            );
        }

        for (key, mutation) in router.mutations.store {
            // mutation.ty.key = format!("{}{}", prefix, key);
            self.mutations.append(
                format!("{}{}", prefix, key),
                self.middleware.build(mutation.exec),
                mutation.ty,
            );
        }

        for (key, subscription) in router.subscriptions.store {
            // subscription.ty.key = format!("{}{}", prefix, key);
            self.subscriptions.append(
                format!("{}{}", prefix, key),
                self.middleware.build(subscription.exec),
                subscription.ty,
            );
        }

        self.type_map.extend(&router.type_map);

        self
    }

    /// `legacy_merge` maintains the `merge` functionality prior to release 0.1.3
    /// It will flow the `TMiddleware` and `TCtx` out of the child router to the parent router.
    /// This was a confusing behavior and is generally not useful so it has been deprecated.
    ///
    /// This function will be remove in a future release. If you are using it open a GitHub issue to discuss your use case and longer term solutions for it.
    pub fn legacy_merge<TNewLayerCtx, TIncomingMiddleware>(
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
            config,
            middleware,
            mut queries,
            mut mutations,
            mut subscriptions,
            mut type_map,
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

        type_map.extend(&router.type_map);

        RouterBuilder {
            config,
            middleware: MiddlewareMerger {
                middleware,
                middleware2: router.middleware,
                phantom: PhantomData,
            },
            queries,
            mutations,
            subscriptions,
            type_map,
            phantom: PhantomData,
        }
    }

    pub fn build(self) -> Router<TCtx, TMeta> {
        let Self {
            config,
            queries,
            mutations,
            subscriptions,
            type_map,
            ..
        } = self;

        let export_path = config.export_bindings_on_build.clone();
        let router = Router {
            config,
            queries,
            mutations,
            subscriptions,
            type_map,
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
