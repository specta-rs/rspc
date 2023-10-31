use std::future::Future;

use rspc_core::internal::{IntoMiddlewareResult, MiddlewareContext};

// `TNewCtx` is sadly require to constain the impl at the bottom of this file. If you can remove it your a god.
pub trait MiddlewareFn<TLCtx, TNewCtx>:
    Fn(MiddlewareContext<TNewCtx>, TLCtx) -> Self::Fut + Send + Sync + 'static
where
    TLCtx: Send + Sync + 'static,
{
    type Fut: Future<Output = Self::Result> + Send + 'static;
    type Result: IntoMiddlewareResult;

    fn execute(&self, ctx: TLCtx, mw: MiddlewareContext<TNewCtx>) -> Self::Fut;
}

impl<TLCtx, TNewCtx, F, Fu> MiddlewareFn<TLCtx, TNewCtx> for F
where
    TLCtx: Send + Sync + 'static,
    F: Fn(MiddlewareContext<TNewCtx>, TLCtx) -> Fu + Send + Sync + 'static,
    Fu: Future + Send + 'static,
    Fu::Output: IntoMiddlewareResult + Send + 'static,
{
    type Fut = Fu;
    type Result = Fu::Output;

    fn execute(&self, ctx: TLCtx, mw: MiddlewareContext<TNewCtx>) -> Self::Fut {
        self(mw, ctx)
    }
}
