use std::{
    any::{type_name, Any},
    fmt,
    sync::Arc,
};

use serde::Deserializer;

use crate::{DynInput, ProcedureStream};

// TODO: Document the importance of the `size_hint`

/// a single type-erased operation that the server can execute.
///
/// TODO: Show constructing and executing procedure.
pub struct Procedure<TCtx> {
    handler: Arc<dyn Fn(TCtx, DynInput) -> ProcedureStream + Send + Sync>,
}

impl<TCtx> Procedure<TCtx> {
    pub fn new(
        handler: impl Fn(TCtx, DynInput) -> ProcedureStream + Send + Sync + 'static,
    ) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }

    pub fn exec_with_deserializer<'de, D: Deserializer<'de> + Send>(
        &self,
        ctx: TCtx,
        input: D,
    ) -> ProcedureStream {
        let mut deserializer = <dyn erased_serde::Deserializer>::erase(input);
        let value = DynInput {
            value: None,
            deserializer: Some(&mut deserializer),
            type_name: type_name::<D>(),
        };

        (self.handler)(ctx, value)
    }

    pub fn exec_with_value<T: Any + Send>(&self, ctx: TCtx, input: T) -> ProcedureStream {
        let mut input = Some(input);
        let value = DynInput {
            value: Some(&mut input),
            deserializer: None,
            type_name: type_name::<T>(),
        };

        (self.handler)(ctx, value)
    }
}

impl<TCtx> Clone for Procedure<TCtx> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
        }
    }
}

impl<TCtx> fmt::Debug for Procedure<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}