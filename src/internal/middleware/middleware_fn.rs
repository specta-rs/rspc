use std::future::Future;

use super::{IntoMiddlewareResult, MiddlewareContext, TODOTemporaryOnlyValidMarker};

// TODO: Maybe use a trait + struct w/ trait to erase the `M` marker from the rest of the system.

// `TNewCtx` is sadly require to constrain the impl at the bottom of this file. If you can remove it your a god.
pub trait MiddlewareFn<TLCtx, TNewCtx>:
    Fn(MiddlewareContext<TNewCtx>, TLCtx) -> Self::Fut + Send + Sync + 'static
where
    TLCtx: Send + Sync + 'static,
{
    type Fut: Future<Output = Self::Result> + Send + 'static;
    type Result: IntoMiddlewareResult<TODOTemporaryOnlyValidMarker>;
    type NewCtx;

    fn execute(&self, ctx: TLCtx, mw: MiddlewareContext<Self::NewCtx>) -> Self::Fut;
}

impl<TLCtx, TNewCtx, F, Fu> MiddlewareFn<TLCtx, TNewCtx> for F
where
    TLCtx: Send + Sync + 'static,
    F: Fn(MiddlewareContext<TNewCtx>, TLCtx) -> Fu + Send + Sync + 'static,
    Fu: Future + Send + 'static,
    Fu::Output: IntoMiddlewareResult<TODOTemporaryOnlyValidMarker> + Send + 'static,
{
    type Fut = Fu;
    type Result = Fu::Output;
    type NewCtx = TNewCtx;

    fn execute(&self, ctx: TLCtx, mw: MiddlewareContext<Self::NewCtx>) -> Self::Fut {
        self(mw, ctx)
    }
}
