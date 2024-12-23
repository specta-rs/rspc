use std::{
    any::type_name,
    fmt,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::Arc,
};

use serde::Deserializer;

use crate::{DynInput, ProcedureError, ProcedureStream};

// TODO: Discuss cancellation safety

/// a single type-erased operation that the server can execute.
///
/// TODO: Show constructing and executing procedure.
pub struct Procedure<TCtx> {
    handler: Arc<dyn Fn(TCtx, DynInput) -> ProcedureStream + Send + Sync>,

    #[cfg(debug_assertions)]
    handler_name: &'static str,
}

impl<TCtx> Procedure<TCtx> {
    pub fn new<F: Fn(TCtx, DynInput) -> ProcedureStream + Send + Sync + 'static>(
        handler: F,
    ) -> Self {
        Self {
            handler: Arc::new(handler),
            #[cfg(debug_assertions)]
            handler_name: type_name::<F>(),
        }
    }

    pub fn exec(&self, ctx: TCtx, input: DynInput) -> ProcedureStream {
        let (Ok(v) | Err(v)) = catch_unwind(AssertUnwindSafe(|| (self.handler)(ctx, input)))
            .map_err(|err| ProcedureError::Unwind(err).into());
        v
    }

    pub fn exec_with_deserializer<'de, D: Deserializer<'de> + Send>(
        &self,
        ctx: TCtx,
        input: D,
    ) -> ProcedureStream {
        let mut deserializer = <dyn erased_serde::Deserializer>::erase(input);
        let value = DynInput::new_deserializer(&mut deserializer);

        let (Ok(v) | Err(v)) = catch_unwind(AssertUnwindSafe(|| (self.handler)(ctx, value)))
            .map_err(|err| ProcedureError::Unwind(err).into());
        v
    }

    pub fn exec_with_value<T: Send + 'static>(&self, ctx: TCtx, input: T) -> ProcedureStream {
        let mut input = Some(input);
        let value = DynInput::new_value(&mut input);

        let (Ok(v) | Err(v)) = catch_unwind(AssertUnwindSafe(|| (self.handler)(ctx, value)))
            .map_err(|err| ProcedureError::Unwind(err).into());
        v
    }
}

impl<TCtx> Clone for Procedure<TCtx> {
    fn clone(&self) -> Self {
        Self {
            handler: self.handler.clone(),
            #[cfg(debug_assertions)]
            handler_name: self.handler_name,
        }
    }
}

impl<TCtx> fmt::Debug for Procedure<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut t = f.debug_tuple("Procedure");
        #[cfg(debug_assertions)]
        let t = t.field(&self.handler_name);
        t.finish()
    }
}
