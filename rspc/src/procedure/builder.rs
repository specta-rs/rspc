use std::{fmt, future::Future, marker::PhantomData};

use futures::{FutureExt, StreamExt};

use super::{Procedure, ProcedureExecInput, ResolverInput, ResolverOutput};

// TODO: Should these be public so they can be used in middleware? If so document them.
// We hide the generics from the public API so we can change them without a major.
mod sealed {
    use super::*;
    pub struct GG<R, I>(PhantomData<(R, I)>);
}
use futures::Stream;
pub use sealed::GG;

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
    pub fn query<F>(&self, handler: impl Fn(TCtx, I) -> F + 'static) -> Procedure<TCtx>
    where
        F: Future<Output = R> + Send + 'static,
        I: ResolverInput + 'static,
        R: ResolverOutput,
    {
        // TODO: The return type here is wrong. It needs TNewCtx
        Procedure {
            handler: Box::new(move |ctx, input| {
                R::into_procedure_stream(
                    handler(
                        ctx,
                        // TODO: Invalid input error
                        I::from_value(ProcedureExecInput::new(input)).unwrap(),
                    )
                    .into_stream(),
                )
            }),
        }
    }

    pub fn mutation<F>(&self, handler: impl Fn(TCtx, I) -> F + 'static) -> Procedure<TCtx>
    where
        F: Future<Output = R> + Send + 'static,
        I: ResolverInput + 'static,
        R: ResolverOutput,
    {
        // TODO: The return type here is wrong. It needs TNewCtx
        Procedure {
            handler: Box::new(move |ctx, input| {
                R::into_procedure_stream(
                    handler(
                        ctx,
                        // TODO: Invalid input error
                        I::from_value(ProcedureExecInput::new(input)).unwrap(),
                    )
                    .into_stream(),
                )
            }),
        }
    }

    pub fn subscription<F, S>(&self, handler: impl Fn(TCtx, I) -> F + 'static) -> Procedure<TCtx>
    where
        F: Future<Output = S> + Send + 'static,
        S: Stream<Item = R> + Send + 'static,
        I: ResolverInput + 'static,
        R: ResolverOutput,
    {
        // TODO: The return type here is wrong. It needs TNewCtx
        Procedure {
            handler: Box::new(move |ctx, input| {
                R::into_procedure_stream(
                    handler(
                        ctx,
                        // TODO: Invalid input error
                        I::from_value(ProcedureExecInput::new(input)).unwrap(),
                    )
                    .into_stream()
                    .flatten(),
                )
            }),
        }
    }
}
