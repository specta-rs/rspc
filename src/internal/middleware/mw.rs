use std::future::Future;

use crate::middleware_from_core::{MiddlewareContext, MwV2Result};

pub trait Middleware<TLCtx>: Send + Sync + 'static {
    type Fut: Future<Output = Self::Result> + Send + 'static;
    type Result: MwV2Result<Ctx = Self::NewCtx>;
    type NewCtx: Send + Sync + 'static;

    // TODO: Seal this method & possibly some of the assoicated types
    // TODO: Rename
    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext) -> Self::Fut;
}

impl<TLCtx, F, Fu> Middleware<TLCtx> for F
where
    TLCtx: Send + Sync + 'static,
    F: Fn(MiddlewareContext, TLCtx) -> Fu + Send + Sync + 'static,
    Fu: Future + Send + 'static,
    Fu::Output: MwV2Result + Send + 'static,
{
    type Fut = Fu;
    type Result = Fu::Output;
    type NewCtx = <Fu::Output as MwV2Result>::Ctx; // TODO: Make this work with context switching

    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext) -> Self::Fut {
        self(mw, ctx)
    }
}
