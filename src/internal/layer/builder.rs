use std::marker::PhantomData;

use crate::internal::middleware::MiddlewareFn;

use super::{Layer, MiddlewareLayer};

/// TODO
pub trait LayerBuilder: Send + Sync + 'static {
    type Ctx: Send + Sync + 'static;
    type LayerCtx: Send + Sync + 'static;

    type LayerResult<T>: Layer<Self::Ctx>
    where
        T: Layer<Self::LayerCtx>;

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerCtx>;
}

/// Is responsible for joining together two layers.
/// A layer could be a middleware or a resolver.
pub struct MiddlewareLayerBuilder<TMiddleware, TNewMiddleware, M> {
    pub(crate) middleware: TMiddleware,
    pub(crate) mw: TNewMiddleware,
    pub(crate) phantom: PhantomData<M>,
}

impl<TMiddleware, TNewCtx, TNewMiddleware> LayerBuilder
    for MiddlewareLayerBuilder<TMiddleware, TNewMiddleware, TNewCtx>
where
    TNewCtx: Send + Sync + 'static,
    TMiddleware: LayerBuilder + Send + Sync + 'static,
    TNewMiddleware: MiddlewareFn<TMiddleware::LayerCtx, TNewCtx> + Send + Sync + 'static,
{
    type Ctx = TMiddleware::Ctx;
    type LayerCtx = TNewCtx;
    type LayerResult<T> = TMiddleware::LayerResult<MiddlewareLayer<TMiddleware::LayerCtx, TNewCtx, T, TNewMiddleware>>
        where
            T: Layer<Self::LayerCtx>;

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerCtx> + Sync,
    {
        self.middleware.build(MiddlewareLayer {
            next,
            mw: self.mw,
            phantom: PhantomData,
        })
    }
}
