use std::{any::Any, fmt, marker::PhantomData, sync::Arc};

use crate::{middleware::middleware::MiddlewareHandler, procedure::ProcedureMeta};

pub(crate) struct NextInner {
    // TODO: `pub(super)` over `pub(crate)`
    pub(crate) meta: ProcedureMeta,
    // TODO: This holds: MiddlewareHandler<TCtx, TInput, TReturn>
    pub(crate) next: Arc<dyn Any + Send + Sync>,
}

pub struct Next<TCtx, TInput, TReturn> {
    // TODO: `pub(super)` over `pub(crate)`
    pub(crate) inner: NextInner,
    pub(crate) phantom: PhantomData<(TCtx, TInput, TReturn)>,
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
        self.inner.meta.clone()
    }

    pub async fn exec(self, ctx: TCtx, input: TInput) -> TReturn {
        let handler = self
            .inner
            .next
            .downcast_ref::<MiddlewareHandler<TCtx, TInput, TReturn>>()
            .expect("bruh");

        handler(ctx, input, ProcedureMeta {}).await
    }
}
