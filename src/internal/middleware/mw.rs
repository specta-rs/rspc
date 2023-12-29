use std::future::Future;

use super::{MiddlewareContext, MwV2Result};

// TODO: Move to only `MiddlewareFn`
pub trait Middleware<TLCtx, TNewCtx>: Send + Sync + 'static {
    type Fut: Future<Output = Self::Result> + Send + 'static;
    type Result: MwV2Result<Ctx = Self::NewCtx>;
    type NewCtx: Send + Sync + 'static;

    // TODO: Seal this method & possibly some of the assoicated types
    // TODO: Rename
    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext<TNewCtx>) -> Self::Fut;
}

impl<TLCtx, TNewCtx, F, Fu> Middleware<TLCtx, TNewCtx> for F
where
    TLCtx: Send + Sync + 'static,
    TNewCtx: Send + Sync + 'static,
    F: Fn(MiddlewareContext<TNewCtx>, TLCtx) -> Fu + Send + Sync + 'static,
    Fu: Future + Send + 'static,
    Fu::Output: MwV2Result + Send + 'static,
{
    type Fut = Fu;
    type Result = Fu::Output;
    type NewCtx = <Fu::Output as MwV2Result>::Ctx;

    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext<TNewCtx>) -> Self::Fut {
        self(mw, ctx)
    }
}
