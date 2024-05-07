use std::{fmt, future::Future, marker::PhantomData};

use super::{Output, Procedure};

// TODO: Should these be public so they can be used in middleware? If so document them.
// We hide the generics from the public API so we can change them without a major.
mod sealed {
    use super::*;
    pub struct GG<R, I>(PhantomData<(R, I)>);
}
pub use sealed::GG;
use serde::de::DeserializeOwned;

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

// TODO: Backwards or forwards infer the format - forwards in annoying but default generics are a thing???
impl<TCtx, R, I> ProcedureBuilder<TCtx, GG<R, I>> {
    /// TODO
    pub fn query<F>(&self, handler: impl Fn(TCtx, I) -> F + 'static) -> Procedure<TCtx>
    where
        F: Future<Output = R> + 'static,
        I: DeserializeOwned,
        R: Output,
    {
        // TODO: The return type here is wrong. It needs TNewCtx
        Procedure {
            // TODO: Error handling
            handler: Box::new(move |ctx, input| {
                // TODO: Spawn onto runtime without returning boxed but does that actually make a performance difference???
                let result = handler(ctx, erased_serde::deserialize(input).unwrap());
                // let t: R = todo!(); // TODO: Async
                // F::exec(t)

                todo!();
            }),
        }
    }
}
