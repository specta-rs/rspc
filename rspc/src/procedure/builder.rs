use std::{error, fmt, future::Future, marker::PhantomData};

use futures::{FutureExt, Stream, StreamExt};
use specta::{DataType, TypeDefs};

use crate::{
    middleware::{Middleware, MiddlewareInner},
    procedure::ProcedureMeta,
};

use super::{Procedure, ProcedureExecInput, ProcedureType, ResolverInput, ResolverOutput};

// TODO: Document the generics like `Middleware`
pub struct ProcedureBuilder<TErr, TCtx, TNextCtx, TInput, TResult> {
    pub(super) mw: Option<MiddlewareInner<TNextCtx, TInput, TResult>>, // TODO: Should this have a default instead of an `Option`???
    pub(super) input: Option<fn(&mut TypeDefs) -> DataType>,
    pub(super) phantom: PhantomData<(TErr, TCtx)>,
}

impl<TCtx, TErr: error::Error, TNextCtx, TInput, TResult> fmt::Debug
    for ProcedureBuilder<TErr, TCtx, TNextCtx, TInput, TResult>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

// TODO: The double usage of `TCtx` in multiple parts of this impl block is plain wrong and will break context switching
impl<TCtx, TErr: error::Error, TInput, TResult> ProcedureBuilder<TErr, TCtx, TCtx, TInput, TResult>
where
    TCtx: 'static,
{
    pub fn error<TNewErr: error::Error>(
        self,
    ) -> ProcedureBuilder<TNewErr, TCtx, TCtx, TInput, TResult> {
        ProcedureBuilder {
            mw: self.mw,
            input: self.input,
            phantom: PhantomData,
        }
    }

    pub fn with<TNextCtx, I, R>(
        self,
        mw: Middleware<TErr, TCtx, TInput, TResult, TNextCtx, I, R>,
    ) -> ProcedureBuilder<TErr, TCtx, TCtx, I, R> {
        ProcedureBuilder {
            // TODO: Merge in the previous middleware with the incoming middleware
            mw: todo!(), // Some(mw.inner),
            input: self.input,
            phantom: PhantomData,
        }
    }

    pub fn query<F, M>(self, handler: impl Fn(TCtx, TInput) -> F + 'static) -> Procedure<TCtx, TErr>
    where
        F: Future<Output = TResult> + Send + 'static,
        TInput: ResolverInput,
        TResult: ResolverOutput<M, TErr> + 'static,
    {
        Procedure {
            input: self.input.unwrap_or(TInput::data_type),
            ty: ProcedureType::Query,
            result: TResult::data_type,
            handler: Box::new(move |ctx, input| {
                let f = (self.mw.as_ref().unwrap().handler)(
                    ctx,
                    TInput::from_value(ProcedureExecInput::new(input))?,
                    ProcedureMeta {},
                );

                // handler(ctx, input).into_stream(),

                Ok(TResult::into_procedure_stream(f.into_stream()))
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
