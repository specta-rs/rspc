use std::future::Future;

use serde::de::DeserializeOwned;
use specta::Type;

use crate::internal::middleware::{MiddlewareContext, MwV2Result};

/// TODO
///
// TODO: Maybe move this specific type out of `internal` given I think it's used in the public API.
pub trait Middleware<TLCtx>: SealedMiddleware<TLCtx> {}

/// TODO
///
// TODO: Maybe move this specific type out of `internal` given I think it's used in the public API.
pub trait ConstrainedMiddleware<TLCtx>:
    Middleware<TLCtx> + Fn(MiddlewareContext, TLCtx) -> Self::Fut + Send + Sync + 'static
where
    TLCtx: Send + Sync + 'static,
{
}

impl<TLCtx, F, Fu> ConstrainedMiddleware<TLCtx> for F
where
    TLCtx: Send + Sync + 'static,
    F: Fn(MiddlewareContext, TLCtx) -> Fu + Send + Sync + 'static,
    Fu: Future + Send + 'static,
    Fu::Output: MwV2Result + Send + 'static,
{
}

mod private {
    use super::*;

    pub trait SealedMiddleware<TLCtx>: Send + Sync + 'static {
        type Fut: Future<Output = Self::Result> + Send + 'static;
        type Result: MwV2Result<Ctx = Self::NewCtx>;
        type NewCtx: Send + Sync + 'static;
        type Arg<T: Type + DeserializeOwned + 'static>: Type + DeserializeOwned + 'static;

        // TODO: Rename
        fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext) -> Self::Fut;
    }

    impl<TLCtx, T: SealedMiddleware<TLCtx>> Middleware<TLCtx> for T {}

    impl<TLCtx, F, Fu> SealedMiddleware<TLCtx> for F
    where
        TLCtx: Send + Sync + 'static,
        F: Fn(MiddlewareContext, TLCtx) -> Fu + Send + Sync + 'static,
        Fu: Future + Send + 'static,
        Fu::Output: MwV2Result + Send + 'static,
    {
        type Fut = Fu;
        type Result = Fu::Output;
        type NewCtx = <Fu::Output as MwV2Result>::Ctx; // TODO: Make this work with context switching
        type Arg<T: Type + DeserializeOwned + 'static> = T;

        fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext) -> Self::Fut {
            self(mw, ctx)
        }
    }
}

#[cfg(feature = "unstable")]
pub use private::SealedMiddleware;

#[cfg(not(feature = "unstable"))]
pub(crate) use private::SealedMiddleware;
