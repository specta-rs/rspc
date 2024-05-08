use std::{fmt, future::Future, marker::PhantomData};

use crate::procedure::{ProcedureExecResult, ProcedureExecResultFuture};

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

impl<TCtx, R, I> ProcedureBuilder<TCtx, GG<R, I>> {
    /// TODO
    pub fn query<F>(&self, handler: impl Fn(TCtx, I) -> F + 'static) -> Procedure<TCtx>
    where
        F: Future<Output = R> + Send + 'static,
        I: DeserializeOwned,
        R: Output,
        I: 'static,
    {
        // TODO: The return type here is wrong. It needs TNewCtx
        Procedure {
            handler: Box::new(move |ctx, input| {
                let input: I = match input.to_value() {
                    Some(v) => v.unwrap().deserialize_into().unwrap(),
                    None => input
                        .to_option_dyn_any()
                        .downcast_mut::<Option<I>>()
                        .unwrap() // TODO: Invalid input error
                        .take()
                        .expect("value already taken"),
                };

                let fut = handler(ctx, input);
                ProcedureExecResult::Future(ProcedureExecResultFuture::new(async move {
                    fut.await.into_result()
                }))
            }),
        }
    }
}
