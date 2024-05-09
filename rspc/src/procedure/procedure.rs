use std::{fmt, marker::PhantomData};

use super::{
    builder::GG,
    procedure_input::{AnyInput, InputValueInner},
    stream::ProcedureStream,
    Argument, ProcedureBuilder,
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

    pub fn exec<'de, T: Argument<'de>>(&self, ctx: TCtx, input: T) -> ProcedureStream {
        match input.into_deserializer() {
            Ok(deserializer) => {
                let mut input = <dyn erased_serde::Deserializer>::erase(deserializer);
                (self.handler)(ctx, &mut input)
            }
            Err(input) => (self.handler)(ctx, &mut AnyInput(Some(input.into_value()))),
        }
    }
}
