// TODO: Remove `middleware/base.rs` if this works
// TODO: Optionally box procedure

use std::marker::PhantomData;

use crate::{internal::middleware::MiddlewareBuilder, layer::Layer};

pub struct ResolverBuilder<TCtx>(pub(crate) PhantomData<TCtx>);

impl<TCtx> MiddlewareBuilder for ResolverBuilder<TCtx>
where
    TCtx: Send + Sync + 'static,
{
    type Ctx = TCtx;
    type LayerCtx = TCtx;
    type LayerResult<T> = T
    where
        T: Layer<Self::LayerCtx>;

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerCtx>,
    {
        todo!();
    }
}
