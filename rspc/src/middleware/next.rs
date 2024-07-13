use std::{fmt, sync::Arc};

use crate::{middleware::middleware::MiddlewareHandler, procedure::ProcedureMeta};

pub struct Next<TCtx, TInput, TReturn> {
    // TODO: `pub(super)` over `pub(crate)`
    pub(crate) meta: ProcedureMeta,
    pub(crate) next: Arc<MiddlewareHandler<TCtx, TInput, TReturn>>,
}

impl<TCtx, TInput, TReturn> fmt::Debug for Next<TCtx, TInput, TReturn> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Next").finish()
    }
}

// TODO: Constrain these generics to the required stuff
impl<TCtx, TInput, TReturn> Next<TCtx, TInput, TReturn>
where
    TCtx: 'static,
    TInput: 'static,
    TReturn: 'static,
{
    pub fn meta(&self) -> ProcedureMeta {
        self.meta.clone()
    }

    pub async fn exec(self, ctx: TCtx, input: TInput) -> TReturn {
        (self.next)(ctx, input, ProcedureMeta {}).await
    }
}
