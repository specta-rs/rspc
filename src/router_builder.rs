use std::{marker::PhantomData, sync::Arc};

use futures::{Future, Stream};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use specta::{Type, TypeDefs};

use crate::{
    internal::{
        BaseMiddleware, BuiltProcedureBuilder, Demo, LayerResult, Middleware, MiddlewareBuilder,
        MiddlewareContext, ProcedureStore, UnbuiltProcedureBuilder,
    },
    Config, DoubleArgStreamMarker, ExecError, RequestLayer, Resolver, Router, StreamResolver,
};

pub struct RouterBuilder<
    TCtx = (), // The is the context the current router was initialised with
    TMeta = (),
    TMiddleware = BaseMiddleware<TCtx>,
> where
    TCtx: 'static,
    TMiddleware: MiddlewareBuilder<TCtx>,
{
    config: Config,
    middleware: TMiddleware,
    queries: ProcedureStore<TCtx>,
    mutations: ProcedureStore<TCtx>,
    subscriptions: ProcedureStore<TCtx>,
    typ_store: TypeDefs,
    phantom: PhantomData<TMeta>,
}

impl<TCtx, TMeta> Router<TCtx, TMeta>
where
    TCtx: Send + 'static,
    TMeta: Send + Sync + 'static,
{
    pub fn new() -> RouterBuilder<TCtx, TMeta, BaseMiddleware<TCtx>> {
        RouterBuilder {
            config: Config::new(),
            middleware: BaseMiddleware::new(),
            queries: ProcedureStore::new("query"),
            mutations: ProcedureStore::new("mutation"),
            subscriptions: ProcedureStore::new("subscription"),
            typ_store: TypeDefs::new(),
            phantom: PhantomData,
        }
    }
}

impl<TCtx, TMeta> RouterBuilder<TCtx, TMeta>
where
    TCtx: Send + 'static,
    TMeta: Send + Sync + 'static,
{
    pub fn new() -> RouterBuilder<TCtx, TMeta, BaseMiddleware<TCtx>> {
        RouterBuilder {
            config: Config::new(),
            middleware: BaseMiddleware::new(),
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
    TLayerCtx: 'static,
    TMiddleware: MiddlewareBuilder<TCtx, LayerContext = TLayerCtx> + 'static,
{
    /// Attach a configuration to the router. Calling this multiple times will overwrite the previous config.
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn middleware<TNewLayerCtx, TFut>(
        self,
        func: fn(MiddlewareContext<TLayerCtx, TNewLayerCtx>) -> TFut,
    ) -> RouterBuilder<TCtx, TMeta, Demo<TCtx, TNewLayerCtx>>
    where
        TNewLayerCtx: Send + 'static,
        TFut: Future<Output = Result<Value, ExecError>> + Send + 'static,
    {
        let Self {
            config,
            middleware,
            queries,
            mutations,
            subscriptions,
            typ_store,
            ..
        } = self;

        RouterBuilder {
            config,
            middleware: Demo {
                bruh: Box::new(move |nextmw: Box<dyn Middleware<TNewLayerCtx>>| {
                    // TODO: An `Arc` is more avoid than should be need but it's probs better than leaking memory.
                    // I can't work out lifetimes to avoid this but would be great to try again!
                    let nextmw = Arc::new(nextmw);
                    middleware.build(Box::new(move |ctx, arg, (kind, key)| {
                        Ok(LayerResult::FutureStreamOrValue(Box::pin(func(
                            MiddlewareContext::<TLayerCtx, TNewLayerCtx> {
                                key,
                                kind,
                                ctx,
                                arg,
                                nextmw: nextmw.clone(),
                            },
                        ))))
                    }))
                }),
            },
            queries,
            mutations,
            subscriptions,
            typ_store,
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
        let resolver = builder(UnbuiltProcedureBuilder::new()).resolver;
        self.queries.append(
            key.into(),
            self.middleware.build(Box::new(move |nextmw, arg, _| {
                resolver.exec(
                    nextmw,
                    serde_json::from_value(arg).map_err(ExecError::DeserializingArgErr)?,
                )
            })),
            TResolver::typedef(&mut self.typ_store),
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
        let resolver = builder(UnbuiltProcedureBuilder::new()).resolver;
        self.mutations.append(
            key.into(),
            self.middleware.build(Box::new(move |nextmw, arg, _| {
                resolver.exec(
                    nextmw,
                    serde_json::from_value(arg).map_err(ExecError::DeserializingArgErr)?,
                )
            })),
            TResolver::typedef(&mut self.typ_store),
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
        let resolver = builder(UnbuiltProcedureBuilder::new()).resolver;
        self.subscriptions.append(
            key.into(),
            self.middleware.build(Box::new(move |nextmw, arg, _| {
                resolver.exec(
                    nextmw,
                    serde_json::from_value(arg).map_err(ExecError::DeserializingArgErr)?,
                )
            })),
            TResolver::typedef(&mut self.typ_store),
        );
        self
    }

    pub fn merge<TNewLayerCtx, TIncomingMiddleware>(
        self,
        prefix: &'static str,
        router: RouterBuilder<TLayerCtx, TMeta, TIncomingMiddleware>,
    ) -> RouterBuilder<TCtx, TMeta, Demo<TCtx, TNewLayerCtx>>
    where
        TIncomingMiddleware: MiddlewareBuilder<TLayerCtx, LayerContext = TNewLayerCtx> + 'static,
    {
        if prefix == "" || prefix.starts_with("rpc.") {
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
            config,
            middleware: Demo {
                bruh: Box::new(move |next: Box<dyn Middleware<TNewLayerCtx>>| {
                    middleware.build(router.middleware.build(next))
                }),
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
        if let Some(export_path) = export_path {
            router.export_ts(export_path).unwrap();
        }

        router
    }
}
