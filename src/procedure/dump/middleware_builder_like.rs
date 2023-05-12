use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use specta::Type;

use crate::{
    internal::{Layer, Middleware, MiddlewareBuilderLikeCompat},
    MiddlewareLayer,
};

// TODO: Trying to remove this?
pub trait MiddlewareBuilderLike: Send + 'static {
    type Ctx: Send + Sync + 'static;
    type LayerCtx: Send + Sync + 'static;
    type Arg<T: Type + DeserializeOwned + 'static>: Type + DeserializeOwned + 'static;

    type LayerResult<T>: Layer<Self::Ctx>
    where
        T: Layer<Self::LayerCtx>;

    fn build<T>(self, next: T) -> Self::LayerResult<T>
    where
        T: Layer<Self::LayerCtx>;
}

impl<M: MiddlewareBuilderLike> MiddlewareBuilderLikeCompat for M {
    type Arg<T: Type + DeserializeOwned + 'static> = M::Arg<T>;
}

pub struct MiddlewareLayerBuilder<TMiddleware, TNewMiddleware>
where
    TMiddleware: MiddlewareBuilderLike,
    TNewMiddleware: Middleware<TMiddleware::LayerCtx>,
{
    pub(crate) middleware: TMiddleware,
    pub(crate) mw: TNewMiddleware,
}

impl<TLayerCtx, TMiddleware, TNewMiddleware> MiddlewareBuilderLike
    for MiddlewareLayerBuilder<TMiddleware, TNewMiddleware>
where
    TLayerCtx: Send + Sync + 'static,
    TMiddleware: MiddlewareBuilderLike<LayerCtx = TLayerCtx> + Send + Sync + 'static,
    TNewMiddleware: Middleware<TLayerCtx> + Send + Sync + 'static,
{
    type Ctx = TMiddleware::Ctx;
    type LayerCtx = TNewMiddleware::NewCtx;
    type LayerResult<T> = TMiddleware::LayerResult<MiddlewareLayer<TLayerCtx, T, TNewMiddleware>>
    where
        T: Layer<Self::LayerCtx>;
    type Arg<T: Type + DeserializeOwned + 'static> = TNewMiddleware::Arg<T>;

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
