use std::{fmt, future::Future};

use futures::FutureExt;

use crate::{
    middleware::{Middleware, MiddlewareHandler},
    Error, State,
};

use super::{ProcedureKind, ProcedureMeta, UnbuiltProcedure};

// TODO: Document the generics like `Middleware`
pub struct ProcedureBuilder<TError, TCtx, TNextCtx, TInput, TResult> {
    pub(super) build: Box<
        dyn FnOnce(
            ProcedureKind,
            Vec<Box<dyn FnOnce(&mut State, ProcedureMeta) + 'static>>,
            MiddlewareHandler<TError, TNextCtx, TInput, TResult>,
        ) -> UnbuiltProcedure<TCtx>,
    >,
}

impl<TCtx, TError, TNextCtx, TInput, TResult> fmt::Debug
    for ProcedureBuilder<TError, TCtx, TNextCtx, TInput, TResult>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

impl<TRootCtx, TCtx, TError, TInput, TResult>
    ProcedureBuilder<TError, TRootCtx, TCtx, TInput, TResult>
where
    TError: Error,
    TRootCtx: 'static,
    TCtx: 'static,
    TInput: 'static,
    TResult: 'static,
{
    pub fn with<TNextCtx, I, R>(
        self,
        mw: Middleware<TError, TCtx, TInput, TResult, TNextCtx, I, R>,
    ) -> ProcedureBuilder<TError, TRootCtx, TNextCtx, I, R>
    where
        TNextCtx: 'static,
        I: 'static,
        R: 'static,
    {
        ProcedureBuilder {
            build: Box::new(|ty, mut setups, handler| {
                if let Some(setup) = mw.setup {
                    setups.push(setup);
                }

                (self.build)(ty, setups, (mw.inner)(handler))
            }),
        }
    }

    pub fn setup(self, func: impl FnOnce(&mut State, ProcedureMeta) + 'static) -> Self {
        Self {
            build: Box::new(|ty, mut setups, handler| {
                setups.push(Box::new(func));
                (self.build)(ty, setups, handler)
            }),
        }
    }

    pub fn query<F: Future<Output = Result<TResult, TError>> + Send + 'static>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
    ) -> UnbuiltProcedure<TRootCtx> {
        (self.build)(
            ProcedureKind::Query,
            Vec::new(),
            Box::new(move |ctx, input, _| Box::pin(handler(ctx, input))),
        )
    }

    pub fn mutation<F: Future<Output = Result<TResult, TError>> + Send + 'static>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
    ) -> UnbuiltProcedure<TRootCtx> {
        (self.build)(
            ProcedureKind::Mutation,
            Vec::new(),
            Box::new(move |ctx, input, _| Box::pin(handler(ctx, input))),
        )
    }
}

impl<TRootCtx, TCtx, TError, TInput, S, T>
    ProcedureBuilder<TError, TRootCtx, TCtx, TInput, crate::Stream<S>>
where
    TError: Error,
    TRootCtx: 'static,
    TCtx: 'static,
    TInput: 'static,
    S: futures::Stream<Item = Result<T, TError>> + Send + 'static,
{
    pub fn subscription<F: Future<Output = Result<S, TError>> + Send + 'static>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
    ) -> UnbuiltProcedure<TRootCtx> {
        (self.build)(
            ProcedureKind::Subscription,
            Vec::new(),
            Box::new(move |ctx, input, _| {
                Box::pin(handler(ctx, input).map(|s| s.map(|s| crate::Stream(s))))
            }),
        )
    }
}
