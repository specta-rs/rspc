use std::{fmt, future::Future, marker::PhantomData};

use crate::middleware::MiddlewareBuilder;

use super::{Next, Procedure};

// TODO: Should these be public so they can be used in middleware? If so document them.
// We hide the generics from the public API so we can change them without a major.
mod sealed {
    use super::*;
    pub struct GG<R, I>(PhantomData<(R, I)>);
}
pub use sealed::GG;

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
    /// TODO
    pub fn with<NextR, NextI, TNextCtx>(
        &self,
        mw: MiddlewareBuilder<TCtx, I, Next<NextR, NextI, TNextCtx>, R>,
    ) -> ProcedureBuilder<TNextCtx, GG<NextR, NextI>> {
        todo!();
    }

    // pub fn with<F, NextR, NextI, TNextCtx>(
    //     &self,
    //     handler: impl Fn(TCtx, I, Next<NextR, NextI, TNextCtx>) -> F,
    // ) -> ProcedureBuilder<TNextCtx, GG<NextR, NextI>>
    // where
    //     F: Future<Output = R>,
    // {
    //     todo!();
    // }

    /// TODO
    pub fn query<F: Future<Output = R>>(&self, handler: impl Fn(TCtx, I) -> F) -> Procedure<TCtx> {
        // TODO: The return type here is wrong. It needs TNewCtx
        todo!();
    }
}
