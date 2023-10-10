use std::future::Future;

use serde::de::DeserializeOwned;
use specta::Type;

use rspc_core::internal::{MiddlewareContext, MwV2Result};

// `TNewCtx` is sadly require to constain the impl at the bottom of this file. If you can remove it your a god.
pub trait MiddlewareFn<TLCtx, TNewCtx>:
    Fn(MiddlewareContext<TNewCtx>, TLCtx) -> Self::Fut + Send + Sync + 'static
where
    TLCtx: Send + Sync + 'static,
{
    type Fut: Future<Output = Self::Result> + Send + 'static;
    type Result: MwV2Result;

    // TODO: Rename
    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext<TNewCtx>) -> Self::Fut;
}

impl<TLCtx, TNewCtx, F, Fu> MiddlewareFn<TLCtx, TNewCtx> for F
where
    TLCtx: Send + Sync + 'static,
    F: Fn(MiddlewareContext<TNewCtx>, TLCtx) -> Fu + Send + Sync + 'static,
    Fu: Future + Send + 'static,
    Fu::Output: MwV2Result + Send + 'static,
{
    type Fut = Fu;
    type Result = Fu::Output;

    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext<TNewCtx>) -> Self::Fut {
        self(mw, ctx)
    }
}
