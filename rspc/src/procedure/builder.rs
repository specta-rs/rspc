use std::{error, fmt, future::Future, marker::PhantomData};

use futures::Stream;
use specta::{DataType, TypeDefs};

use crate::{
    middleware::{Middleware, MiddlewareInner},
    procedure::ProcedureMeta,
    Infallible,
};

use super::{mw::Mw, Procedure, ProcedureType, ResolverInput, ResolverOutput};

// TODO: Document the generics like `Middleware`
pub struct ProcedureBuilder<TErr: error::Error, TCtx, TNextCtx, TInput, TResult> {
    pub(super) mw: Mw<TErr, TCtx, TNextCtx, TInput, TResult>,
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

// We enforce this can only be called once.
// This is because switching it would require us to track the initial type or erased it. // TODO: Clarify this
impl<TCtx, TNewCtx, TInput, TResult> ProcedureBuilder<Infallible, TCtx, TNewCtx, TInput, TResult>
where
    TCtx: 'static,
{
    pub fn error<TNewErr: error::Error>(
        self,
    ) -> ProcedureBuilder<TNewErr, TCtx, TNewCtx, TInput, TResult> {
        ProcedureBuilder {
            mw: todo!(), // TODO: self.mw,
            input: self.input,
            phantom: PhantomData,
        }
    }
}

// TODO: The double usage of `TCtx` in multiple parts of this impl block is plain wrong and will break context switching
impl<TCtx, TErr, TInput, TResult> ProcedureBuilder<TErr, TCtx, TCtx, TInput, TResult>
where
    TErr: error::Error + 'static,
    TCtx: 'static,
    TInput: 'static,
    TResult: 'static,
{
    pub fn with<TNextCtx, I, R>(
        self,
        mw: Middleware<TErr, TCtx, TInput, TResult, TNextCtx, I, R>,
    ) -> ProcedureBuilder<TErr, TCtx, TCtx, I, R> {
        ProcedureBuilder {
            mw: Mw {
                build: Box::new(|MiddlewareInner { setup, handler }| {
                    if let Some(setup) = setup {
                        setup(todo!(), ProcedureMeta {});
                    }
                    drop(setup);

                    (self.mw.build)(mw.inner)
                }),
            },
            input: self.input,
            phantom: PhantomData,
        }
    }

    pub fn query<F, M>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
    ) -> Procedure<TCtx, TErr>
    where
        F: Future<Output = TResult> + Send + 'static,
        TInput: ResolverInput,
        TResult: ResolverOutput<M, TErr> + 'static,
    {
        Procedure {
            input: self.input.unwrap_or(TInput::data_type),
            ty: ProcedureType::Query,
            result: TResult::data_type,
            handler: (self.mw.build)(MiddlewareInner {
                setup: None,
                handler: Box::new(move |ctx, input, meta| Box::pin(handler(ctx, input))),
            }),
        }
    }

    pub fn mutation<F, M>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
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
            handler: (self.mw.build)(MiddlewareInner {
                setup: None,
                handler: Box::new(move |ctx, input, meta| Box::pin(handler(ctx, input))),
            }),
        }
    }

    pub fn subscription<F, S, M>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
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
            // handler: Box::new(move |ctx, input| {
            //     Ok(TResult::into_procedure_stream(
            //         handler(ctx, TInput::from_value(ProcedureExecInput::new(input))?)
            //             .into_stream()
            //             .flatten(),
            //     ))
            // }),
            // handler: (self.mw.build)(MiddlewareInner {
            //     setup: None,
            //     handler: Box::new(move |ctx, input, meta| Box::pin(handler(ctx, input))),
            // }),
            handler: todo!(),
        }
    }
}
