use std::marker::PhantomData;

use super::{
    arg_mapper::{ArgumentMapper, ArgumentMapperPassthrough},
    ConstrainedMiddleware, SealedMiddleware,
};

use rspc_core::internal::MiddlewareContext;

// TODO: These types need to move out of the `internal` module

// TODO: Rename `Middleware` unpon finsihing `new-mw-system`
pub struct Middleware2<TLCtx, M, ArgMapper> {
    m: M,
    phantom: PhantomData<(TLCtx, ArgMapper)>,
}

impl<TLCtx, M> Middleware2<TLCtx, M, ArgumentMapperPassthrough>
where
    TLCtx: Send + Sync + 'static,
    M: ConstrainedMiddleware<TLCtx>,
{
    pub fn new(m: M) -> Self {
        Self {
            m,
            phantom: PhantomData,
        }
    }
}

impl<TLCtx, M, ArgMapper> Middleware2<TLCtx, M, ArgMapper>
where
    TLCtx: Send + Sync + 'static,
    M: ConstrainedMiddleware<TLCtx>,
    ArgMapper: ArgumentMapper,
{
    pub fn mapper<A: ArgumentMapper>(self) -> Middleware2<TLCtx, M, ArgMapper> {
        Middleware2 {
            m: self.m,
            phantom: PhantomData,
        }
    }
}

impl<TLCtx, M, ArgMapper> SealedMiddleware<TLCtx> for Middleware2<TLCtx, M, ArgMapper>
where
    TLCtx: Send + Sync + 'static,
    M: ConstrainedMiddleware<TLCtx>,
    ArgMapper: ArgumentMapper,
{
    type Fut = M::Fut;
    type Result = M::Result;
    type NewCtx = M::NewCtx;
    type Arg<T: specta::Type + serde::de::DeserializeOwned + 'static> = ArgMapper::Input<T>;

    fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext) -> Self::Fut {
        self.m.run_me(ctx, mw)
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
