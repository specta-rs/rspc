use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use specta::Type;

use crate::internal::{middleware::Middleware, Layer};

pub trait MiddlewareBuilder: private::SealedMiddlewareBuilder {}

mod private {
    use crate::internal::middleware::MiddlewareLayer;

    use super::*;

    pub trait SealedMiddlewareBuilder: Send + 'static {
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

    impl<T: SealedMiddlewareBuilder> MiddlewareBuilder for T {}

    pub struct MiddlewareLayerBuilder<TMiddleware, TNewMiddleware>
    where
        TMiddleware: MiddlewareBuilder,
        TNewMiddleware: Middleware<TMiddleware::LayerCtx>,
    {
        pub(crate) middleware: TMiddleware,
        pub(crate) mw: TNewMiddleware,
    }

    impl<TLayerCtx, TMiddleware, TNewMiddleware> SealedMiddlewareBuilder
        for MiddlewareLayerBuilder<TMiddleware, TNewMiddleware>
    where
        TLayerCtx: Send + Sync + 'static,
        TMiddleware: MiddlewareBuilder<LayerCtx = TLayerCtx> + Send + Sync + 'static,
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
}

pub(crate) use private::{MiddlewareLayerBuilder, SealedMiddlewareBuilder};
