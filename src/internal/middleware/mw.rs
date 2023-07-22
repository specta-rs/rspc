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

impl<TLCtx, F, Fu, R> ConstrainedMiddleware<TLCtx> for F
where
    TLCtx: Send + Sync + 'static,
    F: Fn(MiddlewareContext, TLCtx) -> Fu + Send + Sync + 'static,
    Fu: Future<Output = R> + Send,
    R: MwV2Result + Send,
{
}

// TODO: Private and sealed
// pub trait Demo<'a, TLCtx>:
//     Fn(MiddlewareContext<'a>, TLCtx) -> Self::Fu + Send + Sync + 'static
// {
//     type Fu: Future<Output = Self::R> + Send + 'a;
//     type R: MwV2Result + Send + 'a;
// }

// impl<'a, TLCtx, F, Fu, R> Demo<'a, TLCtx> for F
// where
//     TLCtx: Send + Sync + 'static,
//     F: Fn(MiddlewareContext<'a>, TLCtx) -> Fu + Send + Sync + 'static,
//     Fu: Future<Output = R> + Send + 'a,
//     R: MwV2Result + Send + 'a,
// {
//     type Fu = Fu;
//     type R = R;
// }

// impl<'a, TLCtx, F> ConstrainedMiddleware<TLCtx> for F
// where
//     TLCtx: Send + Sync + 'static,
//     F: Demo<'a, TLCtx>, // Fn(MiddlewareContext<'a>, TLCtx) -> Fu + Send + Sync + 'static,
//                         // Fu: Future<Output = R> + Send + 'a,
//                         // R: MwV2Result + Send + 'a,
// {
// }

mod private {
    use super::*;

    pub trait SealedMiddleware<TLCtx>: Send + Sync + 'static {
        type Fut: Future<Output = Self::Result> + Send;
        type Result: MwV2Result<Ctx = Self::NewCtx>;
        // Technically this can be inferred from `Self::Result` however this is a public API (Used in Mw mapper)
        type NewCtx: Send + Sync + 'static;
        type Arg<T: Type + DeserializeOwned + 'static>: Type + DeserializeOwned + 'static;

        // TODO: Rename
        fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext) -> Self::Fut;
    }

    impl<TLCtx, T: SealedMiddleware<TLCtx>> Middleware<TLCtx> for T {}

    // impl<'a, TLCtx, F> SealedMiddleware<TLCtx> for F
    // where
    //     TLCtx: Send + Sync + 'static,
    //     F: super::Demo<'a, TLCtx>,
    //     // F: Fn(MiddlewareContext, TLCtx) -> Fu + Send + Sync + 'static,
    //     // Fu: Future<Output = R> + Send,
    //     // R: MwV2Result + Send,
    // {
    //     type Fut = F::Fu;
    //     type Result = F::R;
    //     type NewCtx = <F::R as MwV2Result>::Ctx;
    //     type Arg<T: Type + DeserializeOwned + 'static> = T;

    //     fn run_me(&self, ctx: TLCtx, mw: MiddlewareContext) -> Self::Fut {
    //         self(mw, ctx)
    //     }
    // }

    impl<TLCtx, F, Fu, R> SealedMiddleware<TLCtx> for F
    where
        TLCtx: Send + Sync + 'static,
        F: Fn(MiddlewareContext, TLCtx) -> Fu + Send + Sync + 'static,
        Fu: Future<Output = R> + Send,
        R: MwV2Result + Send,
    {
        type Fut = Fu;
        type Result = R;
        type NewCtx = R::Ctx;
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
