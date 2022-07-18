use std::{future::Future, marker::PhantomData};

use futures::{Stream, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::{
    ConcreteArg, Config, Context, ExecError, Key, KeyDefinition, MiddlewareChain, MiddlewareResult,
    Operation, ResolverResult, Router, SubscriptionOperation,
};

pub struct RouterBuilder<
    TCtx = (), // The is the context the current router was initialised with
    TMeta = (),
    TQueryKey = &'static str,
    TMutationKey = &'static str,
    TSubscriptionKey = &'static str,
    TLayerCtx = TCtx, // This is the context of the current layer -> Whatever the last middleware returned
    TRootCtx = (), // This is the first context of the top most router (when joining routers). This is used for middleware currently and will be removed.
> where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
    TLayerCtx: Send + Sync + 'static,
    TRootCtx: Send + Sync + 'static,
{
    config: Config,
    middleware: MiddlewareChain<TCtx, TLayerCtx>,
    query: Operation<TQueryKey, TCtx>,
    mutation: Operation<TMutationKey, TCtx>,
    subscription: SubscriptionOperation<TSubscriptionKey, TRootCtx>,
    phantom: PhantomData<(TRootCtx, TMeta)>,
}

impl<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
    Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
{
    pub fn new() -> RouterBuilder<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TCtx, TCtx>
    {
        RouterBuilder {
            config: Config::new(),
            middleware: Box::new(|next| Box::new(move |ctx, args| next(ctx, args))),
            query: Operation::new("query"),
            mutation: Operation::new("mutation"),
            subscription: SubscriptionOperation::new("subscription"),
            phantom: PhantomData,
        }
    }
}

impl<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TLayerCtx, TRootCtx>
    RouterBuilder<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TLayerCtx, TRootCtx>
where
    TRootCtx: Send + Sync + 'static,
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
    TLayerCtx: Send + Sync + 'static,
{
    /// Attach a configuration to the router. Calling this multiple times will overwrite the previous config.
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn middleware<TNextLayerCtx, TFut>(
        self,
        resolver: fn(
            TLayerCtx,
            Box<dyn FnOnce(TNextLayerCtx) -> Result<MiddlewareResult, ExecError> + Send + Sync>,
        ) -> TFut,
    ) -> RouterBuilder<
        TCtx,
        TMeta,
        TQueryKey,
        TMutationKey,
        TSubscriptionKey,
        TNextLayerCtx,
        TRootCtx,
    >
    where
        TNextLayerCtx: Send + Sync + 'static,
        TFut: Future<Output = Result<Value, ExecError>> + Send + Sync + 'static,
    {
        let Self {
            middleware,
            query,
            mutation,
            subscription,
            ..
        } = self;

        RouterBuilder {
            config: self.config,
            middleware: Box::new(move |next| {
                let next: &'static _ = Box::leak(next); // TODO: Cleanup memory

                (middleware)(Box::new(move |ctx, args| {
                    let y = resolver(ctx, Box::new(move |ctx| next(ctx, args)));
                    Ok(MiddlewareResult::Future(Box::pin(y)))
                }))
            }),
            query: query,
            mutation: mutation,
            subscription: subscription,
            phantom: PhantomData,
        }
    }

    pub fn query<TKey, TArg, TResolverMarker, TResolverResult>(
        mut self,
        key: TKey,
        resolver: fn(Context<TLayerCtx>, TArg) -> TResolverResult,
    ) -> Self
    where
        TKey: Key<TQueryKey, TArg>,
        TArg: DeserializeOwned + TS + 'static,
        TResolverResult: ResolverResult<TResolverMarker> + 'static,
    {
        self.query.insert::<TArg, TResolverMarker, TResolverResult>(
            key.to_val(),
            (self.middleware)(Box::new(move |ctx, arg| {
                let arg = match arg {
                    ConcreteArg::Value(v) => {
                        serde_json::from_value(v).map_err(ExecError::ErrDeserialiseArg)?
                    }
                    ConcreteArg::Unknown(v) => *v
                        .downcast::<TArg>()
                        .map_err(|_| ExecError::UnreachableInternalState)?,
                };
                resolver(Context { ctx }, arg).into_middleware_result()
            })),
        );
        self
    }

    pub fn mutation<TKey, TArg, TResolverMarker, TResolverResult>(
        mut self,
        key: TKey,
        resolver: fn(Context<TLayerCtx>, TArg) -> TResolverResult,
    ) -> Self
    where
        TKey: Key<TMutationKey, TArg>,
        TArg: DeserializeOwned + TS + 'static,
        TResolverResult: ResolverResult<TResolverMarker> + 'static,
    {
        self.mutation
            .insert::<TArg, TResolverMarker, TResolverResult>(
                key.to_val(),
                (self.middleware)(Box::new(move |ctx, arg| {
                    let arg = match arg {
                        ConcreteArg::Value(v) => {
                            serde_json::from_value(v).map_err(ExecError::ErrDeserialiseArg)?
                        }
                        ConcreteArg::Unknown(v) => *v
                            .downcast::<TArg>()
                            .map_err(|_| ExecError::UnreachableInternalState)?,
                    };
                    resolver(Context { ctx }, arg).into_middleware_result()
                })),
            );
        self
    }

    pub fn subscription<TKey, TArg, TResolverMarker, TStream>(
        mut self,
        key: TKey,
        resolver: fn(
            Context<TRootCtx>, // TODO: Should be TLayerCtx when middleware support added
            TArg,
        ) -> TStream,
    ) -> Self
    where
        TKey: Key<TSubscriptionKey, TArg>,
        TArg: DeserializeOwned + TS + Send + Sync + 'static,
        TStream: Stream + Send + Sync + 'static,
        <TStream as Stream>::Item: ResolverResult<TResolverMarker> + Serialize + 'static,
    {
        self.subscription
            .insert::<TArg, TResolverMarker, <TStream as Stream>::Item>(
                key.to_val(),
                Box::new(move |ctx, arg| {
                    let arg = match arg {
                        ConcreteArg::Value(v) => {
                            serde_json::from_value(v).map_err(ExecError::ErrDeserialiseArg)?
                        }
                        ConcreteArg::Unknown(v) => *v
                            .downcast::<TArg>()
                            .map_err(|_| ExecError::UnreachableInternalState)?,
                    };

                    Ok(resolver(Context { ctx }, arg)
                        .map(|v| serde_json::to_value(v).map_err(ExecError::ErrSerialiseResult))
                        .boxed())
                }),
            );
        self
    }

    pub fn merge<TLayerCtx2>(
        self,
        prefix: &'static str,
        router: RouterBuilder<
            TLayerCtx,
            TMeta,
            TQueryKey,
            TMutationKey,
            TSubscriptionKey,
            TLayerCtx2,
            TRootCtx,
        >,
    ) -> RouterBuilder<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TLayerCtx2, TRootCtx>
    where
        TLayerCtx2: Send + Sync + 'static,
    {
        let Self {
            middleware,
            mut query,
            mut mutation,
            mut subscription,
            ..
        } = self;

        let (operations, type_defs) = router.query.consume();
        for (key, operation) in operations {
            query.insert_internal(
                TQueryKey::add_prefix(key, prefix),
                (middleware)(Box::new(operation)),
            );
        }
        query.insert_typedefs(
            type_defs
                .into_iter()
                .map(|(key, value)| (TQueryKey::add_prefix(key, prefix), value))
                .collect(),
        );

        let (operations, type_defs) = router.mutation.consume();
        for (key, operation) in operations {
            mutation.insert_internal(
                TMutationKey::add_prefix(key, prefix),
                (middleware)(Box::new(operation)),
            );
        }
        mutation.insert_typedefs(
            type_defs
                .into_iter()
                .map(|(key, value)| (TMutationKey::add_prefix(key, prefix), value))
                .collect(),
        );

        let (operations, type_defs) = router.subscription.consume();
        for (key, operation) in operations {
            subscription.insert_internal(TSubscriptionKey::add_prefix(key, prefix), operation);
        }
        subscription.insert_typedefs(
            type_defs
                .into_iter()
                .map(|(key, value)| (TSubscriptionKey::add_prefix(key, prefix), value))
                .collect(),
        );

        let router_middleware: &'static _ = Box::leak(router.middleware); // TODO: Cleanup memory
        RouterBuilder {
            config: self.config,
            middleware: Box::new(move |next| middleware((router_middleware)(next))),
            query: query,
            mutation: mutation,
            subscription: subscription,
            phantom: PhantomData,
        }
    }

    pub fn build(self) -> Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TRootCtx> {
        let Self {
            query,
            mutation,
            subscription,
            ..
        } = self;

        // TODO: Validate all enum variants have been assigned a value

        let router = Router {
            query: query,
            mutation: mutation,
            subscription: subscription,
            phantom: PhantomData,
        };

        #[cfg(debug_assertions)]
        if let Some(export_path) = self.config.export_bindings_on_build {
            router.export_ts(export_path).unwrap();
        }

        router
    }
}
