use std::{fmt, future::Future, marker::PhantomData};

use crate::playground::{Middleware, NextTrait};

use super::{Next, Procedure};

// TODO: Should these be public so they can be used in middleware? If so document them.
// We hide the generics from the public API so we can change them without a major.
mod sealed {
    use super::*;
    pub struct GG<R, I>(PhantomData<(R, I)>);
}
pub use sealed::GG;
use serde::{de::DeserializeOwned, Deserialize};

/// TODO
// TODO: Maybe default generics
pub struct ProcedureBuilder<TCtx, G> {
    pub(super) phantom: PhantomData<(TCtx, G)>,
}

impl<TCtx, G> fmt::Debug for ProcedureBuilder<TCtx, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

impl<TCtx, R, I> ProcedureBuilder<TCtx, GG<R, I>> {
    // /// TODO
    // pub fn with<NextR, NextI, TNextCtx>(
    //     &self,
    //     mw: MiddlewareBuilder<TCtx, I, Next<NextR, NextI, TNextCtx>, R>,
    // ) -> ProcedureBuilder<TNextCtx, GG<NextR, NextI>> {
    //     todo!();
    // }

    // /// TODO
    // pub fn with<NextR, NextI, TNextCtx, F, M>(
    //     &self,
    //     mw: M,
    // ) -> ProcedureBuilder<TNextCtx, GG<NextR, NextI>>
    // where
    //     M: Fn(TCtx, I, Next<NextR, NextI, TNextCtx>) -> F,
    //     F: Future<Output = R>,
    // {
    //     todo!();
    // }

    // /// TODO
    // pub fn register<NextR, NextI, TNextCtx, F, M>(
    //     &self,
    //     init: impl FnOnce(()) -> M, // TODO: Context type for input
    // ) -> ProcedureBuilder<TNextCtx, GG<NextR, NextI>>
    // where
    //     M: Fn(TCtx, I, Next<NextR, NextI, TNextCtx>) -> F,
    //     F: Future<Output = R>,
    // {
    //     todo!();
    // }

    // pub fn with<F, NextR, NextI, TNextCtx>(
    //     &self,
    //     handler: impl Fn(TCtx, I, Next<NextR, NextI, TNextCtx>) -> F,
    // ) -> ProcedureBuilder<TNextCtx, GG<NextR, NextI>>
    // where
    //     F: Future<Output = R>,
    // {
    //     todo!();
    // }

    // TODO: Defer the generic checking here????
    /// TODO
    pub fn with<NextR, N: NextTrait, M: Middleware<N>>(
        &self,
        mw: M,
        // TODO: Don't hardcode `R`
    ) -> ProcedureBuilder<M::Ctx, GG<i32, M::Input>> {
        todo!();
    }

    /// TODO
    pub fn query<F>(&self, handler: impl Fn(TCtx, I) -> F + 'static) -> Procedure<TCtx>
    where
        F: Future<Output = R> + 'static,
        I: DeserializeOwned,
    {
        // TODO: The return type here is wrong. It needs TNewCtx
        Procedure {
            // TODO: Error handling
            handler: Box::new(move |ctx, input| {
                // TODO: Spawn onto runtime without returning boxed but does that actually make a performance difference???
                let result = handler(ctx, erased_serde::deserialize(input).unwrap()).await;
            }),
        }
    }
}
