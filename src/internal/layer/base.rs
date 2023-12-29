use std::marker::PhantomData;

use crate::{internal::layer::LayerBuilder, layer::Layer};

pub struct BaseLayer<TCtx>(PhantomData<TCtx>);

impl<TCtx> Default for BaseLayer<TCtx> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<TCtx> LayerBuilder for BaseLayer<TCtx>
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
        next
    }
}
