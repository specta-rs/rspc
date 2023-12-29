use std::marker::PhantomData;

use crate::{internal::middleware::Middleware, layer::Layer};

use super::MiddlewareLayer;

// TODO: Can this be made completely internal?
#[doc(hidden)]
pub(crate) trait MiddlewareBuilder: Send + Sync + 'static {
    type Ctx: Send + Sync + 'static;
    type LayerCtx: Send + Sync + 'static;

    type LayerResult<T>: Layer<Self::Ctx>
    where
        T: Layer<Self::LayerCtx>;

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerCtx>;
}

pub struct MiddlewareLayerBuilder<TMiddleware, TNewMiddleware> {
    pub(crate) middleware: TMiddleware,
    pub(crate) mw: TNewMiddleware,
}

impl<TMiddleware, TNewMiddleware> MiddlewareBuilder
    for MiddlewareLayerBuilder<TMiddleware, TNewMiddleware>
where
    TMiddleware: MiddlewareBuilder + Send + Sync + 'static,
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
