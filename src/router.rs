use std::{future::Future, marker::PhantomData};

use serde::de::DeserializeOwned;
use serde_json::Value;
use ts_rs::TS;

use crate::{
    CompiledRouter, ConcreteArg, Context, Key, KeyDefinition, MiddlewareChain, MiddlewareResult,
    Operation, ResolverResult,
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
    subscription: Operation<TSubscriptionKey, TCtx>,
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
            subscription: Operation::new("subscription"),
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
            Box<dyn FnOnce(TNextLayerCtx) -> MiddlewareResult + Send + Sync>,
        ) -> TFut,
    ) -> Router<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey, TNextLayerCtx>
    where
        TNextLayerCtx: Send + Sync + 'static,
        TFut: Future<Output = Value> + Send + Sync + 'static,
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
                    MiddlewareResult::Future(Box::pin(y))
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
        self.query.insert(
            key.to_val(),
            (self.middleware)(Box::new(move |ctx, arg| {
                let arg = match arg {
                    ConcreteArg::Value(v) => serde_json::from_value(v).unwrap(),
                    ConcreteArg::Unknown(v) => *v.downcast::<TArg>().unwrap(),
                };
                resolver(Context { ctx }, arg)
                    .into_middleware_result()
                    .unwrap()
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
        self.mutation.insert(
            key.to_val(),
            (self.middleware)(Box::new(move |ctx, arg| {
                let arg = match arg {
                    ConcreteArg::Value(v) => serde_json::from_value(v).unwrap(),
                    ConcreteArg::Unknown(v) => *v.downcast::<TArg>().unwrap(),
                };
                resolver(Context { ctx }, arg)
                    .into_middleware_result()
                    .unwrap()
            })),
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
            mutation,
            subscription,
            ..
        } = self;

        let operations = router.query.operations();
        for (key, operation) in operations {
            query.insert(
                TQueryKey::add_prefix(key, prefix),
                (middleware)(Box::new(operation)),
            );
        }

        Router {
            middleware: Box::new(move |next| {
                (middleware)(Box::new(move |ctx, args| unimplemented!())) // TODO: This probs shouldn't be unimplemented
            }),
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

        Self::_build(query, mutation, subscription)
    }

    fn _build(
        query: Operation<TQueryKey, TCtx>,
        mutation: Operation<TMutationKey, TCtx>,
        subscription: Operation<TSubscriptionKey, TCtx>,
    ) -> CompiledRouter<TCtx, TMeta, TQueryKey, TMutationKey, TSubscriptionKey> {
        // TODO: Validate all enum variants have been assigned a value

        CompiledRouter {
            query: query,
            mutation: mutation,
            subscription: subscription,
            phantom: PhantomData,
        }
    }
}
