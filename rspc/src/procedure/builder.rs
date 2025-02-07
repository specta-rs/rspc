use std::{fmt, future::Future, marker::PhantomData, sync::Arc};

use crate::{
    middleware::{IntoMiddleware, MiddlewareHandler},
    Error, Procedure,
};

use super::{ErasedProcedure, ProcedureKind, ProcedureMeta};

use futures_util::{FutureExt, Stream};
use rspc_procedure::State;

// TODO: Document the generics like `Middleware`. What order should they be in?
pub struct ProcedureBuilder<TError, TBaseCtx, TCtx, TBaseInput, TInput, TBaseResult, TResult> {
    pub(crate) build: Box<
        dyn FnOnce(
            ProcedureKind,
            Vec<Box<dyn FnOnce(&mut State, ProcedureMeta) + 'static>>,
            MiddlewareHandler<TError, TCtx, TInput, TResult>,
        ) -> ErasedProcedure<TBaseCtx>,
    >,
    pub(crate) phantom: PhantomData<(TBaseInput, TBaseResult)>,
}

impl<TBaseCtx, TError, TCtx, TBaseInput, TInput, TBaseResult, TResult> fmt::Debug
    for ProcedureBuilder<TError, TBaseCtx, TCtx, TBaseInput, TInput, TBaseResult, TResult>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

impl<TRootCtx, TCtx, TError, TBaseInput, TInput, TBaseResult, TResult>
    ProcedureBuilder<TError, TRootCtx, TCtx, TBaseInput, TInput, TBaseResult, TResult>
where
    TError: Error,
    TRootCtx: 'static,
    TCtx: 'static,
    TInput: 'static,
    TBaseInput: 'static,
    TResult: 'static,
    TBaseResult: 'static,
{
    pub fn with<
        M: IntoMiddleware<TError, TRootCtx, TCtx, TBaseInput, TInput, TBaseResult, TResult>,
    >(
        self,
        mw: M,
    ) -> ProcedureBuilder<TError, TRootCtx, M::TNextCtx, TBaseInput, M::I, TBaseResult, M::R>
// where
    //     TNextCtx: 'static,
    //     I: 'static,
    //     R: 'static,
    {
        mw.build(self)
    }

    pub fn setup(self, func: impl FnOnce(&mut State, ProcedureMeta) + 'static) -> Self {
        Self {
            build: Box::new(|ty, mut setups, handler| {
                setups.push(Box::new(func));
                (self.build)(ty, setups, handler)
            }),
            phantom: PhantomData,
        }
    }

    pub fn query<F: Future<Output = Result<TResult, TError>> + Send + 'static>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
    ) -> Procedure<TRootCtx, TBaseInput, TBaseResult> {
        Procedure {
            build: Box::new(move |setups| {
                (self.build)(
                    ProcedureKind::Query,
                    setups,
                    Arc::new(move |ctx, input, _| Box::pin(handler(ctx, input))),
                )
            }),
            phantom: PhantomData,
        }
    }

    pub fn mutation<F: Future<Output = Result<TResult, TError>> + Send + 'static>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
    ) -> Procedure<TRootCtx, TBaseInput, TBaseResult> {
        Procedure {
            build: Box::new(move |setups| {
                (self.build)(
                    ProcedureKind::Mutation,
                    setups,
                    Arc::new(move |ctx, input, _| Box::pin(handler(ctx, input))),
                )
            }),
            phantom: PhantomData,
        }
    }
}

impl<TRootCtx, TCtx, TError, TBaseInput, TInput, TBaseResult, TResult, S>
    ProcedureBuilder<TError, TRootCtx, TCtx, TBaseInput, TInput, TBaseResult, crate::Stream<S>>
where
    TError: Error,
    TRootCtx: 'static,
    TCtx: 'static,
    TInput: 'static,
    TBaseInput: 'static,
    TResult: 'static,
    TBaseResult: 'static,
    S: Stream<Item = TResult> + 'static,
{
    pub fn subscription<F: Future<Output = Result<S, TError>> + Send + 'static>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
    ) -> Procedure<TRootCtx, TBaseInput, crate::Stream<S>> {
        Procedure {
            build: Box::new(move |setups| {
                (self.build)(
                    ProcedureKind::Subscription,
                    setups,
                    Arc::new(move |ctx, input, _| {
                        Box::pin(handler(ctx, input).map(|f| f.map(|s| crate::Stream(s))))
                    }),
                )
            }),
            phantom: PhantomData,
        }
    }
}
