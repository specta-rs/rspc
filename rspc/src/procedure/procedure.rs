use std::{fmt, marker::PhantomData};

use specta::TypeDefs;

use super::{
    builder::GG,
    exec_input::{AnyInput, InputValueInner},
    stream::ProcedureStream,
    ProcedureBuilder, ProcedureInput,
};

/// Represents a single operations on the server that can be executed.
///
/// A [`Procedure`] is built from a [`ProcedureBuilder`] and holds the type information along with the logic to execute the operation.
///
pub struct Procedure<TCtx = ()> {
    pub(super) handler: Box<dyn Fn(TCtx, &mut dyn InputValueInner) -> ProcedureStream>,
}

impl<TCtx> fmt::Debug for Procedure<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

impl<TCtx> Procedure<TCtx> {
    /// Construct a new procedure using [`ProcedureBuilder`].
    pub fn builder<R, I>() -> ProcedureBuilder<TCtx, GG<R, I>> {
        ProcedureBuilder {
            phantom: PhantomData,
        }
    }

    /// Export the [Specta](https://docs.rs/specta) types for this procedure.
    ///
    /// # Usage
    /// ```rust
    /// todo!(); # TODO: Example
    /// ```
    pub fn types(&self, type_map: &mut TypeDefs) {
        todo!();
    }

    /// Execute a procedure with the given context and input.
    ///
    /// This will return a [`ProcedureStream`] which can be used to stream the result of the procedure.
    ///
    /// # Usage
    /// ```rust
    /// use serde_json::Value;
    ///
    /// fn run_procedure(procedure: Procedure) -> Vec<Value> {
    ///     procedure
    ///         .exec((), Value::Null)
    ///         .collect::<Vec<_>>()
    ///         .await
    ///         .into_iter()
    ///         .map(|result| result.serialize(serde_json::value::Serializer).unwrap())
    ///         .collect::<Vec<_>>()
    /// }
    /// ```
    pub fn exec<'de, T: ProcedureInput<'de>>(&self, ctx: TCtx, input: T) -> ProcedureStream {
        match input.into_deserializer() {
            Ok(deserializer) => {
                let mut input = <dyn erased_serde::Deserializer>::erase(deserializer);
                (self.handler)(ctx, &mut input)
            }
            Err(input) => (self.handler)(ctx, &mut AnyInput(Some(input.into_value()))),
        }
    }
}
