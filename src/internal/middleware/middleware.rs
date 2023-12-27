use std::{future::Future, marker::PhantomData};

use crate::middleware_from_core::{MiddlewareContext, MwV2Result};

use super::Middleware;

// TODO: These types need to move out of the `internal` module

// TODO: Rename `Middleware` unpon finsihing `new-mw-system`
pub struct Middleware2<TLCtx, M, Fu> {
    m: M,
    exec: fn(ctx: TLCtx, mw: MiddlewareContext, m: &M) -> Fu,
    phantom: PhantomData<TLCtx>,
}

impl<TLCtx, M, Fu> Middleware<TLCtx> for Middleware2<TLCtx, M, Fu>
where
    TLCtx: Send + Sync + 'static,
    M: Send + Sync + 'static,
    Fu: Future + Send + 'static,
    Fu::Output: MwV2Result,
{
    type Fut = Fu;
    type Result = Fu::Output;
    type NewCtx = <Fu::Output as MwV2Result>::Ctx;

    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext) -> Self::Fut {
        (self.exec)(ctx, mw, &self.m)
    }
}

pub fn mw<TLCtx, M>(m: M) -> Middleware2<TLCtx, M, M::Fut>
where
    TLCtx: Send + Sync + 'static,
    M: Middleware<TLCtx> + Fn(MiddlewareContext, TLCtx) -> M::Fut,
{
    Middleware2 {
        m,
        exec: |ctx, mw, m| m.run_me(ctx, mw),
        phantom: PhantomData,
    }
}
