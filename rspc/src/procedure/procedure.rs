use std::{fmt, marker::PhantomData};

use serde::Deserializer;

use crate::procedure::input_value::AnyInput;

use super::{
    builder::GG, input_value::InputValueInner, stream::ProcedureStream, Argument, ProcedureBuilder,
};

/// TODO
pub struct Procedure<TCtx = ()> {
    pub(super) handler: Box<dyn Fn(TCtx, &mut dyn InputValueInner) -> ProcedureStream>,
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

    // TODO: Can `exec` and `exec_any` be merged into one method

    pub fn exec<'de, 'a: 'de, D: Deserializer<'de> + 'a>(
        &self,
        ctx: TCtx,
        input: D,
    ) -> ProcedureStream {
        (self.handler)(ctx, &mut <dyn erased_serde::Deserializer>::erase(input))
    }

    pub fn exec_any<T: Argument>(&self, ctx: TCtx, input: T) -> ProcedureStream {
        let input = input.into_value();
        (self.handler)(ctx, &mut AnyInput(Some(input)))
    }
}
