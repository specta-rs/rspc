use std::marker::PhantomData;

use crate::procedure::ProcedureMeta;

pub struct Next<TCtx, TInput, TReturn> {
    phantom: PhantomData<(TCtx, TInput, TReturn)>,
}

// TODO: Constrain these generics to the required stuff
impl<TCtx, TInput, TReturn> Next<TCtx, TInput, TReturn> {
    pub fn meta(&self) -> ProcedureMeta {
        todo!();
    }

    pub async fn exec(self, ctx: TCtx, input: TInput) -> TReturn {
        todo!()
    }
}
