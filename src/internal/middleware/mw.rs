use std::future::Future;

use serde::de::DeserializeOwned;
use specta::Type;

use rspc_core::internal::{MiddlewareContext, MwV2Result};

/// TODO
///
// TODO: Maybe move this specific type out of `internal` given I think it's used in the public API.
pub trait Middleware<TLCtx, A: ArgumentMapper>: SealedMiddleware<TLCtx, A> {
    type NewCtx2; // TODO: Rename this
}

/// TODO
///
// TODO: Maybe move this specific type out of `internal` given I think it's used in the public API.
pub trait ConstrainedMiddleware<TLCtx>:
    Middleware<TLCtx, ArgumentMapperPassthrough>
    + Fn(MiddlewareContext<()>, TLCtx) -> Self::Fut
    + Send
    + Sync
    + 'static
where
    TLCtx: Send + Sync + 'static,
{
}

impl<TLCtx, F, Fu> ConstrainedMiddleware<TLCtx> for F
where
    TLCtx: Send + Sync + 'static,
    F: Fn(MiddlewareContext<()>, TLCtx) -> Fu + Send + Sync + 'static,
    Fu: Future + Send + 'static,
    Fu::Output: MwV2Result + Send + 'static,
{
}

mod private {
    use crate::internal::middleware::ArgumentMapper;

    use super::*;

    pub trait SealedMiddleware<TLCtx, A: ArgumentMapper>: Send + Sync + 'static {
        type Fut: Future<Output = Self::Result> + Send + 'static;
        type Result: MwV2Result<Ctx = Self::NewCtx>;
        type NewCtx: Send + Sync + 'static;

        // TODO: Rename
        fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext<A::State>) -> Self::Fut;
    }

    impl<TLCtx, T: SealedMiddleware<TLCtx, A>, A: ArgumentMapper> Middleware<TLCtx, A> for T {
        type NewCtx2 = T::NewCtx;
    }

    impl<TLCtx, F, Fu, A> SealedMiddleware<TLCtx, A> for F
    where
        TLCtx: Send + Sync + 'static,
        F: Fn(MiddlewareContext<A::State>, TLCtx) -> Fu + Send + Sync + 'static,
        Fu: Future + Send + 'static,
        Fu::Output: MwV2Result + Send + 'static,
        A: ArgumentMapper,
    {
        type Fut = Fu;
        type Result = Fu::Output;
        type NewCtx = <Fu::Output as MwV2Result>::Ctx; // TODO: Make this work with context switching

        fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext<A::State>) -> Self::Fut {
            self(mw, ctx)
        }
    }
}

#[cfg(feature = "unstable")]
pub use private::SealedMiddleware;

#[cfg(not(feature = "unstable"))]
pub(crate) use private::SealedMiddleware;

use super::{ArgumentMapper, ArgumentMapperPassthrough};
