use std::{any::TypeId, fmt, marker::PhantomData};

use super::{builder::GG, stream::ProcedureStream, Input, InputValue, ProcedureBuilder};

/// TODO
pub struct Procedure<TCtx = ()> {
    pub(super) handler: Box<dyn Fn(TCtx, InputValue) -> ProcedureStream>,
}

impl<TCtx> fmt::Debug for Procedure<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

impl<TCtx> Procedure<TCtx> {
    pub fn builder<R, I>() -> ProcedureBuilder<TCtx, GG<R, I>> {
        ProcedureBuilder {
            phantom: PhantomData,
        }
    }

    // TODO: Export types

    pub fn exec<I: Input>(&self, ctx: TCtx, input: I) -> ProcedureStream {
        (self.handler)(
            ctx,
            InputValue {
                type_id: TypeId::of::<I>(),
                inner: &mut Some(input),
            },
        )
    }
}
