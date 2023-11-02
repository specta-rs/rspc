use std::{future::Future, marker::PhantomData};

use super::{
    arg_mapper::{ArgumentMapper, ArgumentMapperPassthrough},
    ConstrainedMiddleware, Middleware, SealedMiddleware,
};

use rspc_core::internal::{MiddlewareContext, MwV2Result};

// TODO: These types need to move out of the `internal` module

// TODO: Rename `Middleware` unpon finsihing `new-mw-system`
pub struct Middleware2<TLCtx, M, ArgMapper> {
    m: M,
    phantom: PhantomData<(TLCtx, ArgMapper)>,
}

impl<TLCtx, M, ArgMapper> Middleware2<TLCtx, M, ArgMapper>
where
    TLCtx: Send + Sync + 'static,
{
    pub fn new(m: M) -> Self
    where
        M: ConstrainedMiddleware<TLCtx>,
    {
        Self {
            m,
            phantom: PhantomData,
        }
    }

    pub fn with_mapper<A: ArgumentMapper, Fu, R>(m: M) -> Self
    where
        M: Fn(
                MiddlewareContext<A::State>,
                TLCtx,
                <ArgumentMapperPassthrough as ArgumentMapper>::State,
            ) -> Fu
            + Send
            + Sync
            + 'static,
        Fu: Future + Send + Sync + 'static,
        Fu::Output: MwV2Result<Ctx = TLCtx> + Send + 'static,
    {
        Self {
            m,
            phantom: PhantomData,
        }
    }
}

impl<TLCtx, M, A> SealedMiddleware<TLCtx, A> for Middleware2<TLCtx, M, A>
where
    TLCtx: Send + Sync + 'static,
    M: Middleware<TLCtx, A>,
    A: ArgumentMapper,
{
    type Fut = M::Fut;
    type Result = M::Result;
    type NewCtx = M::NewCtx;

    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext<A::State>) -> Self::Fut {
        // self.m.run_me(ctx, mw)
        todo!();
    }
}

pub fn mw<TLCtx, M>(m: M) -> Middleware2<TLCtx, M, ArgumentMapperPassthrough>
where
    TLCtx: Send + Sync + 'static,
    M: ConstrainedMiddleware<TLCtx>,
{
    Middleware2 {
        m,
        phantom: PhantomData,
    }
}

// TODO: Improve DX of this
pub fn mw_with_arg_mapper<A, TLCtx, M>(m: M) -> Middleware2<TLCtx, M, A>
where
    TLCtx: Send + Sync + 'static,
    M: Middleware<TLCtx, A>
        + Fn(MiddlewareContext<A::State>, TLCtx) -> M::Fut
        + Send
        + Sync
        + 'static,
    A: ArgumentMapper,
{
    Middleware2 {
        m,
        phantom: PhantomData,
    }
}
