use std::{fmt, sync::Arc};

use crate::{
    modern::{middleware::middleware::MiddlewareHandler, procedure::ProcedureMeta},
    State,
};

pub struct Next<TError, TCtx, TInput, TReturn> {
    // TODO: `pub(super)` over `pub(crate)`
    pub(crate) meta: ProcedureMeta,
    pub(crate) next: MiddlewareHandler<TError, TCtx, TInput, TReturn>,
}

impl<TError, TCtx, TInput, TReturn> fmt::Debug for Next<TError, TCtx, TInput, TReturn> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Next").finish()
    }
}

impl<TError, TCtx, TInput, TReturn> Next<TError, TCtx, TInput, TReturn>
where
    TCtx: 'static,
    TInput: 'static,
    TReturn: 'static,
{
    pub fn meta(&self) -> ProcedureMeta {
        self.meta.clone()
    }

    pub async fn exec(&self, ctx: TCtx, input: TInput) -> Result<TReturn, TError> {
        (self.next)(ctx, input, self.meta.clone()).await
    }
}
