use std::{any::Any, fmt};

use serde::Deserializer;

use crate::DynInput;

/// a single type-erased operation that the server can execute.
///
/// TODO: Show constructing and executing procedure.
pub struct Procedure<TCtx> {
    handler: Box<dyn Fn(TCtx, DynInput)>,
}

impl<TCtx> Procedure<TCtx> {
    pub fn new(handler: impl Fn(TCtx, DynInput) + 'static) -> Self {
        Self {
            handler: Box::new(handler),
        }
    }

    pub fn exec_with_deserializer<'de, D: Deserializer<'de>>(&self, ctx: TCtx, input: D) {
        let mut deserializer = <dyn erased_serde::Deserializer>::erase(input);
        let value = DynInput {
            value: None,
            deserializer: Some(&mut deserializer),
        };

        (self.handler)(ctx, value);
    }

    pub fn exec_with_value<T: Any>(&self, ctx: TCtx, input: T) {
        let mut input = Some(input);
        let value = DynInput {
            value: Some(&mut input),
            deserializer: None,
        };

        (self.handler)(ctx, value);
    }
}

impl<TCtx> fmt::Debug for Procedure<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
