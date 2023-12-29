use std::{future::Future, marker::PhantomData};

use super::{Middleware, MiddlewareContext, MwV2Result};

// TODO: These types need to move out of the `internal` module

// TODO: Move to only `MiddlewareFn`
// TODO: Rename `Middleware` unpon finsihing `new-mw-system`
pub struct Middleware2<TLCtx, TNewCtx, M, Fu> {
    m: M,
    exec: fn(ctx: TLCtx, mw: MiddlewareContext<TNewCtx>, m: &M) -> Fu,
    phantom: PhantomData<TLCtx>,
}

impl<TLCtx, TNewCtx, M, Fu> Middleware<TLCtx, TNewCtx> for Middleware2<TLCtx, TNewCtx, M, Fu>
where
    TLCtx: Send + Sync + 'static,
    TNewCtx: Send + Sync + 'static,
    M: Send + Sync + 'static,
    Fu: Future + Send + 'static,
    Fu::Output: MwV2Result,
{
    type Fut = Fu;
    type Result = Fu::Output;
    type NewCtx = <Fu::Output as MwV2Result>::Ctx;

    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext<TNewCtx>) -> Self::Fut {
        (self.exec)(ctx, mw, &self.m)
    }
}

pub fn mw<TLCtx, TNewCtx, M>(m: M) -> Middleware2<TLCtx, TNewCtx, M, M::Fut>
where
    TLCtx: Send + Sync + 'static,
    M: Middleware<TLCtx, TNewCtx> + Fn(MiddlewareContext<TNewCtx>, TLCtx) -> M::Fut,
{
    Middleware2 {
        m,
        exec: |ctx, mw, m| m.run_me(ctx, mw),
        phantom: PhantomData,
    }
}
