use std::{borrow::Cow, marker::PhantomData, panic::Location, sync::Arc};

use futures_util::{FutureExt, TryStreamExt};

use specta::datatype::DataType;

use crate::{
    modern::{
        procedure::{
            ErasedProcedure, ProcedureBuilder, ProcedureMeta, ResolverInput, ResolverOutput,
        },
        Error,
    },
    Extension, ProcedureKind, State,
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
pub struct Procedure<TCtx, TInput, TResult> {
    pub(crate) build:
        Box<dyn FnOnce(Vec<Box<dyn FnOnce(&mut State, ProcedureMeta)>>) -> ErasedProcedure<TCtx>>,
    pub(crate) phantom: PhantomData<(TInput, TResult)>,
}

// TODO: `Debug`, `PartialEq`, `Eq`, `Hash`

impl<TCtx, TInput, TResult> Procedure<TCtx, TInput, TResult> {
    /// Construct a new procedure using [`ProcedureBuilder`].
    #[track_caller]
    pub fn builder<TError>(
    ) -> ProcedureBuilder<TError, TCtx, TCtx, TInput, TInput, TResult, TResult>
    where
        TCtx: Send + 'static,
        TError: Error,
        // Only the first layer (middleware or the procedure) needs to be a valid input/output type
        TInput: ResolverInput,
        TResult: ResolverOutput<TError>,
    {
        let location = Location::caller().clone();
        ProcedureBuilder {
            build: Box::new(move |kind, setup, handler| {
                ErasedProcedure {
                    setup: setup
                        .into_iter()
                        .map(|setup| {
                            let v: Box<dyn FnOnce(&mut State)> =
                                Box::new(move |state: &mut State| {
                                    let key: Cow<'static, str> = "todo".to_string().into(); // TODO: Work this out properly
                                    let meta = ProcedureMeta::new(
                                        key.into(),
                                        kind,
                                        Arc::new(State::default()), // TODO: Can we configure a panic instead of this!
                                    );
                                    setup(state, meta);
                                });
                            v
                        })
                        .collect::<Vec<_>>(),
                    ty: ProcedureType {
                        kind,
                        input: DataType::Any,  // I::data_type(type_map),
                        output: DataType::Any, // R::data_type(type_map),
                        error: DataType::Any,  // TODO
                        location,
                    },
                    inner: Box::new(move |state| {
                        let key: Cow<'static, str> = "todo".to_string().into(); // TODO: Work this out properly
                        let meta = ProcedureMeta::new(key.clone(), kind, state);

                        rspc_procedure::Procedure::new(move |ctx, input| {
                            TResult::into_procedure_stream(
                                handler(
                                    ctx,
                                    TInput::from_input(input).unwrap(), // TODO: Error handling
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
            phantom: PhantomData,
        }
    }

    pub fn with(self, mw: Extension<TCtx, TInput, TResult>) -> Self
    where
        TCtx: 'static,
    {
        Procedure {
            build: Box::new(move |mut setups| {
                if let Some(setup) = mw.setup {
                    setups.push(setup);
                }
                (self.build)(setups)
            }),
            phantom: PhantomData,
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

impl<TCtx, TInput, TResult> Into<ErasedProcedure<TCtx>> for Procedure<TCtx, TInput, TResult> {
    fn into(self) -> ErasedProcedure<TCtx> {
        (self.build)(Default::default())
    }
}
