use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use specta::Type;

use rspc_core::internal::Layer;

// TODO: Deal with sealing properly

// TODO: Can this be made completely internal?
mod private {
    use crate::internal::middleware::{MiddlewareFn, MiddlewareLayer};

    use super::*;

    pub trait MiddlewareBuilder: Send + Sync + 'static {
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

    pub struct MiddlewareLayerBuilder<TMiddleware, TNewMiddleware, TNewCtx> {
        pub(crate) middleware: TMiddleware,
        pub(crate) mw: TNewMiddleware,
        pub(crate) phantom: PhantomData<TNewCtx>,
    }

    impl<TNewCtx, TMiddleware, TNewMiddleware> MiddlewareBuilder
        for MiddlewareLayerBuilder<TMiddleware, TNewMiddleware, TNewCtx>
    where
        TNewCtx: Send + Sync + 'static,
        TMiddleware: MiddlewareBuilder + Send + Sync + 'static,
        TNewMiddleware: MiddlewareFn<TMiddleware::LayerCtx, TNewCtx> + Send + Sync + 'static,
    {
        type Ctx = TMiddleware::Ctx;
        type LayerCtx = TNewCtx;
        type LayerResult<T> = TMiddleware::LayerResult<MiddlewareLayer<TMiddleware::LayerCtx, TNewCtx, T, TNewMiddleware>>
        where
            T: Layer<Self::LayerCtx>;
        type Arg<T: Type + DeserializeOwned + 'static> = T; // TNewMiddleware::Arg<T>;

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

pub(crate) use private::{MiddlewareBuilder, MiddlewareLayerBuilder};
