use std::{borrow::Cow, panic::Location, sync::Arc};

use futures::{FutureExt, TryStreamExt};
use rspc_core::Procedure;
use specta::datatype::DataType;

use crate::{
    modern::{
        procedure::{ProcedureBuilder, ProcedureMeta, ResolverInput, ResolverOutput},
        Error,
    },
    ProcedureKind, State,
};

#[derive(Clone)]
pub(crate) struct ProcedureType {
    pub(crate) kind: ProcedureKind,
    pub(crate) input: DataType,
    pub(crate) output: DataType,
    pub(crate) error: DataType,
    pub(crate) location: Location<'static>,
}

/// Represents a single operations on the server that can be executed.
///
/// A [`Procedure`] is built from a [`ProcedureBuilder`] and holds the type information along with the logic to execute the operation.
///
pub struct Procedure2<TCtx> {
    pub(crate) setup: Vec<Box<dyn FnOnce(&mut State) + 'static>>,
    pub(crate) ty: ProcedureType,
    pub(crate) inner: Box<dyn FnOnce(Arc<State>) -> rspc_core::Procedure<TCtx>>,
}

// TODO: `Debug`, `PartialEq`, `Eq`, `Hash`

impl<TCtx> Procedure2<TCtx> {
    #[cfg(feature = "unstable")]
    /// Construct a new procedure using [`ProcedureBuilder`].
    #[track_caller]
    pub fn builder<I, R, TError>() -> ProcedureBuilder<TError, TCtx, TCtx, I, R>
    where
        TCtx: Send + 'static,
        TError: Error,
        // Only the first layer (middleware or the procedure) needs to be a valid input/output type
        I: ResolverInput,
        R: ResolverOutput<TError>,
    {
        ProcedureBuilder {
            build: Box::new(|kind, setups, handler| {
                Procedure2 {
                    setup: Default::default(),
                    ty: ProcedureType {
                        kind,
                        input: DataType::Any,  // I::data_type(type_map),
                        output: DataType::Any, // R::data_type(type_map),
                        error: DataType::Any,  // TODO
                        location: Location::caller().clone(),
                    },
                    inner: Box::new(move |state| {
                        let key: Cow<'static, str> = "todo".to_string().into(); // TODO: Work this out properly
                        let meta = ProcedureMeta::new(key.clone(), kind, state);

                        Procedure::new(move |ctx, input| {
                            R::into_procedure_stream(
                                handler(
                                    ctx,
                                    I::from_input(input).unwrap(), // TODO: Error handling
                                    meta.clone(),
                                )
                                .into_stream()
                                .map_ok(|v| v.into_stream())
                                .map_err(|err| err.into_resolver_error())
                                .try_flatten()
                                .into_stream(),
                            )
                        })
                    }),
                }
            }),
        }
    }

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
