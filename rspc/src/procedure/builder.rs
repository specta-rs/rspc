use std::{fmt, marker::PhantomData};

use super::Next;

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
        a: impl Fn(TCtx, I, Next<NextR, NextI, TNextCtx>) -> R,
    ) -> ProcedureBuilder<TNextCtx, GG<NextR, NextI>> {
        todo!();
    }

    /// TODO
    pub fn query(&self, f: impl Fn(TCtx, I) -> R) {
        todo!();
    }
}
