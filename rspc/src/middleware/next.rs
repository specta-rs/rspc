use std::{fmt, sync::Arc};

use crate::{middleware::middleware::MiddlewareHandler, procedure::ProcedureMeta};

pub struct Next<TErr, TCtx, TInput, TReturn> {
    // TODO: `pub(super)` over `pub(crate)`
    pub(crate) meta: ProcedureMeta,
    pub(crate) next: Arc<MiddlewareHandler<TErr, TCtx, TInput, TReturn>>,
}

impl<TErr, TCtx, TInput, TReturn> fmt::Debug for Next<TErr, TCtx, TInput, TReturn> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Next").finish()
    }
}

impl<TErr, TCtx, TInput, TReturn> Next<TErr, TCtx, TInput, TReturn>
where
    TCtx: 'static,
    TInput: 'static,
    TReturn: 'static,
{
    pub fn meta(&self) -> ProcedureMeta {
        self.meta.clone()
    }

    pub async fn exec(&self, ctx: TCtx, input: TInput) -> Result<TReturn, TErr> {
        (self.next)(ctx, input, self.meta.clone()).await
    }
}
