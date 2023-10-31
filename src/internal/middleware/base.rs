mod private {
    use std::marker::PhantomData;

    use serde::de::DeserializeOwned;
    use specta::Type;

    use crate::internal::middleware::MiddlewareBuilder;
    use rspc_core::internal::Layer;

    pub struct BaseMiddleware<TCtx>(PhantomData<TCtx>);

    impl<TCtx> Default for BaseMiddleware<TCtx> {
        fn default() -> Self {
            Self(PhantomData)
        }
    }

    impl<TCtx> MiddlewareBuilder for BaseMiddleware<TCtx>
    where
        TCtx: Send + Sync + 'static,
    {
        type Ctx = TCtx;
        type LayerCtx = TCtx;

        type LayerResult<T> = T
        where
            T: Layer<Self::LayerCtx>;
        type Arg<T: Type + DeserializeOwned + 'static> = T;

        fn build<T>(self, next: T) -> Self::LayerResult<T>
        where
            T: Layer<Self::LayerCtx>,
        {
            next
        }
    }
}

pub(crate) use private::BaseMiddleware;
