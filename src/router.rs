use std::{future::Future, marker::PhantomData};

use serde::de::DeserializeOwned;
use serde_json::Value;
use ts_rs::TS;

use crate::{
    CompiledRouter, ConcreteArg, Context, ExecError, Key, KeyDefinition, MiddlewareChain,
    MiddlewareResult, Operation, ResolverResult, SubscriptionContext, SubscriptionOperation,
};

/// TODO
pub struct Router<
    TCtx = (),
    TMeta = (),
    TQueryKey = &'static str,
    TMutationKey = &'static str,
    TSubscriptionKey = &'static str,
    TLayerCtx = TCtx,
> where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
    TLayerCtx: Send + Sync + 'static,
{
    middleware: MiddlewareChain<TCtx, TLayerCtx>,
    query: Operation<TQueryKey, TCtx>,
    mutation: Operation<TMutationKey, TCtx>,
    subscription: SubscriptionOperation<TSubscriptionKey, ()>,
    phantom: PhantomData<TMeta>,
}

// These generics intentionally enforce `TLayerCtx` is initially set to `TCtx`.
impl<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey>
    Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TCtx>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
{
    pub fn new() -> Self {
        Self {
            middleware: Box::new(|next| Box::new(move |ctx, args| next(ctx, args))),
            query: Operation::new("query"),
            mutation: Operation::new("mutation"),
            subscription: SubscriptionOperation::new("subscription"),
            phantom: PhantomData,
        }
    }
}

impl<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TLayerCtx>
    Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TLayerCtx>
where
    TCtx: Send + Sync + 'static,
    TMeta: Send + Sync + 'static,
    TQueryKey: KeyDefinition,
    TMutationKey: KeyDefinition,
    TSubscriptionKey: KeyDefinition,
    TLayerCtx: Send + Sync + 'static,
{
    pub fn middleware<TNextLayerCtx, TFut>(
        self,
        resolver: fn(
            TLayerCtx,
            Box<dyn FnOnce(TNextLayerCtx) -> Result<MiddlewareResult, ExecError> + Send + Sync>,
        ) -> TFut,
    ) -> Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TNextLayerCtx>
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

        Router {
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

    pub fn subscription<TKey, TArg, TResolverMarker, TResolverResult>(
        mut self,
        key: TKey,
        resolver: fn(SubscriptionContext<() /* TODO: TLayerCtx */, TResolverResult>),
    ) -> Self
    where
        TKey: Key<TSubscriptionKey, TArg>,
        TArg: DeserializeOwned + TS + 'static,
        TResolverResult: ResolverResult<TResolverMarker> + 'static,
    {
        self.subscription
            .insert::<TResolverMarker, TResolverResult>(
                key.to_val(),
                Box::new(move |ctx| {
                    resolver(SubscriptionContext {
                        ctx,
                        phantom: PhantomData,
                    });
                }),
            );
        self
    }

    pub fn merge<TLayerCtx2>(
        self,
        prefix: &'static str,
        router: Router<TLayerCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TLayerCtx2>,
    ) -> Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TLayerCtx2>
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
        Router {
            middleware: Box::new(move |next| middleware((router_middleware)(next))),
            query: query,
            mutation: mutation,
            subscription: subscription,
            phantom: PhantomData,
        }
    }

    pub fn build(self) -> CompiledRouter<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey> {
        let Self {
            query,
            mutation,
            subscription,
            ..
        } = self;

        // TODO: Validate all enum variants have been assigned a value

        CompiledRouter {
            query: query,
            mutation: mutation,
            subscription: subscription,
            phantom: PhantomData,
        }
    }
}
