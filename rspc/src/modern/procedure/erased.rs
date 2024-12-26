use std::{borrow::Cow, panic::Location, sync::Arc};

use futures_util::{FutureExt, TryStreamExt};
use rspc_procedure::Procedure;
use specta::datatype::DataType;

use crate::{
    modern::{
        procedure::{ProcedureBuilder, ProcedureMeta, ResolverInput, ResolverOutput},
        Error,
    },
    procedure::ProcedureType,
    State,
};

pub struct ErasedProcedure<TCtx> {
    pub(crate) setup: Vec<Box<dyn FnOnce(&mut State) + 'static>>,
    pub(crate) ty: ProcedureType,
    pub(crate) inner: Box<dyn FnOnce(Arc<State>) -> rspc_procedure::Procedure<TCtx>>,
}

// TODO: `Debug`, `PartialEq`, `Eq`, `Hash`

impl<TCtx> ErasedProcedure<TCtx> {
    // TODO: Expose all fields

    // TODO: Make `pub`
    // pub(crate) fn kind(&self) -> ProcedureKind2 {
    //     self.kind
    // }

    // /// Export the [Specta](https://docs.rs/specta) types for this procedure.
    // ///
    // /// TODO - Use this with `rspc::typescript`
    // ///
    // /// # Usage
    // ///
    // /// ```rust
    // /// todo!(); # TODO: Example
    // /// ```
    // pub fn ty(&self) -> &ProcedureTypeDefinition {
    //     &self.ty
    // }

    // /// Execute a procedure with the given context and input.
    // ///
    // /// This will return a [`ProcedureStream`] which can be used to stream the result of the procedure.
    // ///
    // /// # Usage
    // ///
    // /// ```rust
    // /// use serde_json::Value;
    // ///
    // /// fn run_procedure(procedure: Procedure) -> Vec<Value> {
    // ///     procedure
    // ///         .exec((), Value::Null)
    // ///         .collect::<Vec<_>>()
    // ///         .await
    // ///         .into_iter()
    // ///         .map(|result| result.serialize(serde_json::value::Serializer).unwrap())
    // ///         .collect::<Vec<_>>()
    // /// }
    // /// ```
    // pub fn exec<'de, T: ProcedureInput<'de>>(
    //     &self,
    //     ctx: TCtx,
    //     input: T,
    // ) -> Result<ProcedureStream, InternalError> {
    //     match input.into_deserializer() {
    //         Ok(deserializer) => {
    //             let mut input = <dyn erased_serde::Deserializer>::erase(deserializer);
    //             (self.handler)(ctx, &mut input)
    //         }
    //         Err(input) => (self.handler)(ctx, &mut AnyInput(Some(input.into_value()))),
    //     }
    // }
}
