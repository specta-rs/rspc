use std::marker::PhantomData;

use crate::internal::middleware::Middleware;

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
pub struct MiddlewareLayerBuilder<TMiddleware, TNewMiddleware> {
    pub(crate) middleware: TMiddleware,
    pub(crate) mw: TNewMiddleware,
}

impl<TMiddleware, TNewMiddleware> LayerBuilder
    for MiddlewareLayerBuilder<TMiddleware, TNewMiddleware>
where
    TMiddleware: LayerBuilder + Send + Sync + 'static,
    TNewMiddleware: Middleware<TMiddleware::LayerCtx> + Send + Sync + 'static,
{
    type Ctx = TMiddleware::Ctx;
    type LayerCtx = TNewMiddleware::NewCtx;
    type LayerResult<T> = TMiddleware::LayerResult<MiddlewareLayer<TMiddleware::LayerCtx, T, TNewMiddleware>>
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
