use std::{error, fmt, future::Future, marker::PhantomData};

use futures::{FutureExt, Stream, StreamExt};
use specta::{DataType, TypeDefs};

use crate::middleware::{Middleware, MiddlewareFn};

use super::{Procedure, ProcedureExecInput, ProcedureType, ResolverInput, ResolverOutput};

// TODO: Document the generics like `Middleware`
pub struct ProcedureBuilder<TCtx, TErr, TNextCtx, TInput, TResult> {
    pub(super) mw: Option<MiddlewareFn<TNextCtx>>,
    pub(super) input: Option<fn(&mut TypeDefs) -> DataType>,
    pub(super) phantom: PhantomData<(TCtx, TErr, TNextCtx, TInput, TResult)>,
}

impl<TCtx, TErr: error::Error, TNextCtx, TInput, TResult> fmt::Debug
    for ProcedureBuilder<TCtx, TErr, TNextCtx, TInput, TResult>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

// TODO: The double usage of `TCtx` in multiple parts of this impl block is plain wrong and will break context switching
impl<TCtx, TErr: error::Error, TInput, TResult>
    ProcedureBuilder<TCtx, TErr, TCtx, TInput, TResult>
{
    pub fn error<TNewErr: error::Error>(
        self,
    ) -> ProcedureBuilder<TCtx, TNewErr, TCtx, TInput, TResult> {
        ProcedureBuilder {
            mw: self.mw,
            input: self.input,
            phantom: PhantomData,
        }
    }

    pub fn with<TNextCtx, I, R>(
        self,
        mw: Middleware<TCtx, TErr, I, R, TNextCtx, TInput, TResult>,
    ) -> ProcedureBuilder<TCtx, TErr, TCtx, TInput, TResult> {
        // TODO: Merge in the previous middleware with the incoming middleware
        ProcedureBuilder {
            mw: todo!(),
            input: self.input,
            phantom: PhantomData,
        }
    }

    pub fn query<F, M>(self, handler: impl Fn(TCtx, TInput) -> F + 'static) -> Procedure<TCtx, TErr>
    where
        F: Future<Output = TResult> + Send + 'static,
        TInput: ResolverInput + 'static,
        TResult: ResolverOutput<M, TErr>,
    {
        Procedure {
            input: self.input.unwrap_or(TInput::data_type),
            ty: ProcedureType::Query,
            result: TResult::data_type,
            handler: Box::new(move |ctx, input| {
                Ok(TResult::into_procedure_stream(
                    handler(ctx, TInput::from_value(ProcedureExecInput::new(input))?).into_stream(),
                ))
            }),
        }
    }

    pub fn mutation<F, M>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + 'static,
    ) -> Procedure<TCtx, TErr>
    where
        F: Future<Output = TResult> + Send + 'static,
        TInput: ResolverInput + 'static,
        TResult: ResolverOutput<M, TErr>,
    {
        Procedure {
            input: self.input.unwrap_or(TInput::data_type),
            ty: ProcedureType::Mutation,
            result: TResult::data_type,
            handler: Box::new(move |ctx, input| {
                Ok(TResult::into_procedure_stream(
                    handler(ctx, TInput::from_value(ProcedureExecInput::new(input))?).into_stream(),
                ))
            }),
        }
    }

    pub fn subscription<F, S, M>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + 'static,
    ) -> Procedure<TCtx, TErr>
    where
        F: Future<Output = S> + Send + 'static,
        S: Stream<Item = TResult> + Send + 'static,
        TInput: ResolverInput + 'static,
        TResult: ResolverOutput<M, TErr>,
    {
        Procedure {
            input: self.input.unwrap_or(TInput::data_type),
            ty: ProcedureType::Subscription,
            result: TResult::data_type,
            handler: Box::new(move |ctx, input| {
                Ok(TResult::into_procedure_stream(
                    handler(ctx, TInput::from_value(ProcedureExecInput::new(input))?)
                        .into_stream()
                        .flatten(),
                ))
            }),
        }
    }
}
