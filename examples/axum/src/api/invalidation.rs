//! TODO: Document how this example works.
//!
//! TODO: Expand this example to one which pushes data directly to the frontend.

use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex, PoisonError},
};

use async_stream::stream;
use rspc::middleware::Middleware;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::broadcast;

use super::{BaseProcedure, Context, Router};

#[derive(Clone)]
pub struct Ctx {
    keys: Arc<Mutex<HashMap<String, String>>>,
    tx: broadcast::Sender<InvalidateEvent>,
}

impl Ctx {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            keys: Default::default(),
            tx: broadcast::channel(100).0,
        })
    }
}

#[derive(Debug, Clone, Serialize, Type)]
pub enum InvalidateEvent {
    InvalidateKey(String),
}

#[derive(Deserialize, Type)]
struct SetKeyInput {
    key: String,
    value: String,
}

pub fn mount() -> Router {
    Router::new()
        .procedure("get", {
            <BaseProcedure>::builder()
                // TODO: Why does `TCtx` need a hardcoded type???
                .with(invalidation(|ctx: Context, key, _result| async move {
                    ctx.invalidation
                        .tx
                        .send(InvalidateEvent::InvalidateKey(key))
                        .unwrap();
                }))
                .mutation(|ctx, key: String| async move {
                    let value = ctx
                        .invalidation
                        .keys
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .get(&key)
                        .cloned();

                    Ok(value)
                })
        })
        .procedure("set", {
            <BaseProcedure>::builder().mutation(|ctx, input: SetKeyInput| async move {
                ctx.invalidation
                    .keys
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(input.key, input.value);

                Ok(())
            })
        })
        .procedure("invalidation", {
            // The frontend will subscribe to this for when to invalidate queries.
            <BaseProcedure>::builder().subscription(|ctx, _: ()| async move {
                Ok(stream! {
                    let mut tx = ctx.invalidation.tx.subscribe();
                    while let Ok(msg) = tx.recv().await {
                        yield Ok(msg);
                    }
                })
            })
        })
}

fn invalidation<TError, TCtx, TInput, TResult, F>(
    handler: impl Fn(TCtx, TInput, &Result<TResult, TError>) -> F + Send + Sync + 'static,
) -> Middleware<TError, TCtx, TInput, TResult>
where
    TError: Send + 'static,
    TCtx: Clone + Send + 'static,
    TInput: Clone + Send + 'static,
    TResult: Send + 'static,
    F: Future<Output = ()> + Send + 'static,
{
    let handler = Arc::new(handler);
    Middleware::new(move |ctx: TCtx, input: TInput, next| {
        let handler = handler.clone();
        async move {
            let ctx2 = ctx.clone();
            let input2 = input.clone();
            let result = next.exec(ctx, input).await;
            handler(ctx2, input2, &result).await;
            result
        }
    })
}
