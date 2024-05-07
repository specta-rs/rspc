use std::{fmt, marker::PhantomData};

use super::r#async::ProcedureExecResult;

use super::{builder::GG, ProcedureBuilder};

/// TODO
pub struct Procedure<TCtx = ()> {
    pub(super) handler:
        Box<dyn Fn(TCtx, &mut dyn erased_serde::Deserializer<'_>) -> ProcedureExecResult>,
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

    pub fn exec<I>(&self, ctx: TCtx, input: I) -> ProcedureExecResult {
        (self.handler)(ctx, todo!())
    }
}
