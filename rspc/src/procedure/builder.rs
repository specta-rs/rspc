use std::{error, fmt, future::Future};

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

// TODO: The double usage of `TCtx` in multiple parts of this impl block is plain wrong and will break context switching
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

    // pub fn subscription<F, S, M>(
    //     self,
    //     handler: impl Fn(TNextCtx, TInput) -> F + Send + Sync + 'static,
    // ) -> Procedure<TCtx, TErr>
    // where
    //     F: Future<Output = S> + Send + 'static,
    //     S: Stream<Item = TResult> + Send + 'static,
    //     TInput: ResolverInput,
    //     TResult: ResolverOutput<M, TErr>,
    // {
    //     Procedure {
    //         input: self.input.unwrap_or(TInput::data_type),
    //         ty: ProcedureType::Subscription,
    //         result: TResult::data_type,
    //         // handler: Box::new(move |ctx, input| {
    //         //     Ok(TResult::into_procedure_stream(
    //         //         handler(ctx, TInput::from_value(ProcedureExecInput::new(input))?)
    //         //             .into_stream()
    //         //             .flatten(),
    //         //     ))
    //         // }),
    //         // handler: (self.mw.build)(MiddlewareInner {
    //         //     setup: None,
    //         //     handler: Box::new(move |ctx, input, _| Box::pin(handler(ctx, input))),
    //         // }),
    //         handler: todo!(),
    //     }
    // }
}
