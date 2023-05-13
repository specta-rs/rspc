mod private {
    use std::marker::PhantomData;

    use serde::de::DeserializeOwned;
    use specta::Type;

    use crate::internal::{middleware::SealedMiddlewareBuilder, Layer};

    pub struct BaseMiddleware<TCtx>(PhantomData<TCtx>)
    where
        TCtx: 'static;

    impl<TCtx> Default for BaseMiddleware<TCtx>
    where
        TCtx: 'static,
    {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<TCtx> BaseMiddleware<TCtx>
    where
        TCtx: 'static,
    {
        pub const fn new() -> Self {
            Self(PhantomData)
        }
    }

    impl<TCtx> SealedMiddlewareBuilder for BaseMiddleware<TCtx>
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
