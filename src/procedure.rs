use std::panic::Location;

use specta::datatype::DataType;

use crate::{ProcedureKind, State};

#[derive(Clone)]
pub(crate) struct ProcedureType {
    pub(crate) kind: ProcedureKind,
    pub(crate) input: DataType,
    pub(crate) output: DataType,
    pub(crate) error: DataType,
    // pub(crate) location: Location<'static>,
}

/// Represents a single operations on the server that can be executed.
///
/// A [`Procedure`] is built from a [`ProcedureBuilder`] and holds the type information along with the logic to execute the operation.
///
pub struct Procedure2<TCtx> {
    pub(crate) setup: Vec<Box<dyn FnOnce(&mut State) + 'static>>,
    pub(crate) ty: ProcedureType,
    pub(crate) inner: rspc_core::Procedure<TCtx>,
}

// TODO: `Debug`, `PartialEq`, `Eq`, `Hash`

impl<TCtx> Procedure2<TCtx> {
    // TODO: `fn builder`

    // TODO: Make `pub`
    // pub(crate) fn kind(&self) -> ProcedureKind2 {
    //     self.kind
    // }

    // TODO: Expose all fields

    //     /// Export the [Specta](https://docs.rs/specta) types for this procedure.
    //     ///
    //     /// TODO - Use this with `rspc::typescript`
    //     ///
    //     /// # Usage
    //     ///
    //     /// ```rust
    //     /// todo!(); # TODO: Example
    //     /// ```
    //     pub fn ty(&self) -> &ProcedureTypeDefinition {
    //         &self.ty
    //     }

    //     /// Execute a procedure with the given context and input.
    //     ///
    //     /// This will return a [`ProcedureStream`] which can be used to stream the result of the procedure.
    //     ///
    //     /// # Usage
    //     ///
    //     /// ```rust
    //     /// use serde_json::Value;
    //     ///
    //     /// fn run_procedure(procedure: Procedure) -> Vec<Value> {
    //     ///     procedure
    //     ///         .exec((), Value::Null)
    //     ///         .collect::<Vec<_>>()
    //     ///         .await
    //     ///         .into_iter()
    //     ///         .map(|result| result.serialize(serde_json::value::Serializer).unwrap())
    //     ///         .collect::<Vec<_>>()
    //     /// }
    //     /// ```
    //     pub fn exec<'de, T: ProcedureInput<'de>>(
    //         &self,
    //         ctx: TCtx,
    //         input: T,
    //     ) -> Result<ProcedureStream, InternalError> {
    //         match input.into_deserializer() {
    //             Ok(deserializer) => {
    //                 let mut input = <dyn erased_serde::Deserializer>::erase(deserializer);
    //                 (self.handler)(ctx, &mut input)
    //             }
    //             Err(input) => (self.handler)(ctx, &mut AnyInput(Some(input.into_value()))),
    //         }
    //     }
}
