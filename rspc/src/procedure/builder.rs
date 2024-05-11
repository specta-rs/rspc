use std::{error, fmt, future::Future, marker::PhantomData};

use futures::{FutureExt, StreamExt};
use specta::{DataType, TypeDefs};

use super::{Procedure, ProcedureExecInput, ProcedureType, ResolverInput, ResolverOutput};

// TODO: Should these be public so they can be used in middleware? If so document them.
// We hide the generics from the public API so we can change them without a major.
mod sealed {
    use super::*;
    pub struct GG<C, R, I>(PhantomData<(C, R, I)>);
}
use futures::Stream;
pub use sealed::GG;

// TODO: Maybe default generics
pub struct ProcedureBuilder<TCtx, TErr, G> {
    pub(super) input: Option<fn(&mut TypeDefs) -> DataType>,
    pub(super) phantom: PhantomData<(TCtx, TErr, G)>,
}

impl<TCtx, TErr: error::Error, G> fmt::Debug for ProcedureBuilder<TCtx, TErr, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

impl<TCtx, TErr: error::Error, R, I> ProcedureBuilder<TCtx, TErr, GG<TCtx, R, I>> {
    pub fn query<F, M>(self, handler: impl Fn(TCtx, I) -> F + 'static) -> Procedure<TCtx, TErr>
    where
        F: Future<Output = R> + Send + 'static,
        I: ResolverInput + 'static,
        R: ResolverOutput<M, TErr>,
    {
        Procedure {
            input: self.input.unwrap_or(I::data_type),
            ty: ProcedureType::Query,
            result: R::data_type,
            handler: Box::new(move |ctx, input| {
                Ok(R::into_procedure_stream(
                    handler(ctx, I::from_value(ProcedureExecInput::new(input))?).into_stream(),
                ))
            }),
        }
    }

    pub fn mutation<F, M>(self, handler: impl Fn(TCtx, I) -> F + 'static) -> Procedure<TCtx, TErr>
    where
        F: Future<Output = R> + Send + 'static,
        I: ResolverInput + 'static,
        R: ResolverOutput<M, TErr>,
    {
        Procedure {
            input: self.input.unwrap_or(I::data_type),
            ty: ProcedureType::Mutation,
            result: R::data_type,
            handler: Box::new(move |ctx, input| {
                Ok(R::into_procedure_stream(
                    handler(ctx, I::from_value(ProcedureExecInput::new(input))?).into_stream(),
                ))
            }),
        }
    }

    pub fn subscription<F, S, M>(
        self,
        handler: impl Fn(TCtx, I) -> F + 'static,
    ) -> Procedure<TCtx, TErr>
    where
        F: Future<Output = S> + Send + 'static,
        S: Stream<Item = R> + Send + 'static,
        I: ResolverInput + 'static,
        R: ResolverOutput<M, TErr>,
    {
        Procedure {
            input: self.input.unwrap_or(I::data_type),
            ty: ProcedureType::Subscription,
            result: R::data_type,
            handler: Box::new(move |ctx, input| {
                Ok(R::into_procedure_stream(
                    handler(ctx, I::from_value(ProcedureExecInput::new(input))?)
                        .into_stream()
                        .flatten(),
                ))
            }),
        }
    }
}
