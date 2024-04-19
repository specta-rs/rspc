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
// TODO: Is this a smart name?
pub enum Default {}

/// TODO
// TODO: Maybe default generics
pub struct Procedure<TCtx = (), G = Default> {
    phantom: PhantomData<(G, TCtx)>,
}

impl<TCtx, G> fmt::Debug for Procedure<TCtx, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

impl<TCtx, G> Procedure<TCtx, G> {
    /// TODO
    pub fn new() -> Self {
        todo!();
    }
}

impl<TCtx> Procedure<TCtx, Default> {
    /// TODO
    pub fn with<NextR, NextI, TNextCtx, I, R>(
        &self,
        a: impl Fn(TCtx, I, Next<NextR, NextI, TNextCtx>) -> R,
    ) -> Procedure<TNextCtx, GG<NextR, NextI>> {
        todo!();
    }

    /// TODO
    pub fn query<I, R>(&self, f: impl Fn(TCtx, I) -> R) {
        todo!();
    }
}

impl<TCtx, R, I> Procedure<TCtx, GG<R, I>> {
    /// TODO
    pub fn with<NextR, NextI, TNextCtx>(
        &self,
        a: impl Fn(TCtx, I, Next<NextR, NextI, TNextCtx>) -> R,
    ) -> Procedure<TNextCtx, GG<NextR, NextI>> {
        todo!();
    }

    /// TODO
    pub fn query(&self, f: impl Fn(TCtx, I) -> R) {
        todo!();
    }
}
