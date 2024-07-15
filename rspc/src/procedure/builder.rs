use std::{error, fmt, future::Future};

use futures::FutureExt;

use crate::{
    middleware::{Middleware, MiddlewareHandler},
    State,
};

use super::{Procedure, ProcedureKind, ProcedureMeta};

// TODO: Document the generics like `Middleware`
pub struct ProcedureBuilder<TError, TCtx, TNextCtx, TInput, TResult> {
    pub(super) build: Box<
        dyn FnOnce(
            ProcedureMeta,
            &mut State,
            MiddlewareHandler<TError, TNextCtx, TInput, TResult>,
        ) -> Procedure<TCtx>,
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
    TError: error::Error + Send + 'static,
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
            build: Box::new(|meta, state: &mut State, handler| {
                if let Some(setup) = mw.setup {
                    setup(state, meta.clone());
                }

                (self.build)(meta, state, (mw.inner)(handler))
            }),
        }
    }

    pub fn query<F: Future<Output = Result<TResult, TError>> + Send + 'static>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
    ) -> Procedure<TRootCtx> {
        (self.build)(
            ProcedureMeta::new("todo.todo".into(), ProcedureKind::Query),
            &mut State::default(),
            Box::new(move |ctx, input, _| Box::pin(handler(ctx, input))),
        )
    }

    pub fn mutation<F: Future<Output = Result<TResult, TError>> + Send + 'static>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
    ) -> Procedure<TRootCtx> {
        (self.build)(
            ProcedureMeta::new("todo.todo".into(), ProcedureKind::Mutation),
            &mut State::default(),
            Box::new(move |ctx, input, _| Box::pin(handler(ctx, input))),
        )
    }
}

impl<TRootCtx, TCtx, TError, TInput, S, T>
    ProcedureBuilder<TError, TRootCtx, TCtx, TInput, crate::Stream<S>>
where
    TError: error::Error + Send + 'static,
    TRootCtx: 'static,
    TCtx: 'static,
    TInput: 'static,
    S: futures::Stream<Item = Result<T, TError>> + Send + 'static,
{
    pub fn subscription<F: Future<Output = Result<S, TError>> + Send + 'static>(
        self,
        handler: impl Fn(TCtx, TInput) -> F + Send + Sync + 'static,
    ) -> Procedure<TRootCtx> {
        (self.build)(
            ProcedureMeta::new("todo.todo".into(), ProcedureKind::Mutation),
            &mut State::default(),
            Box::new(move |ctx, input, _| {
                Box::pin(handler(ctx, input).map(|s| s.map(|s| crate::Stream(s))))
            }),
        )
    }
}
