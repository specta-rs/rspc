use std::{borrow::Cow, error, fmt, sync::Arc};

use futures::FutureExt;
use specta::{DataType, TypeDefs};

use super::{
    exec_input::{AnyInput, InputValueInner},
    stream::ProcedureStream,
    InternalError, ProcedureBuilder, ProcedureExecInput, ProcedureInput, ProcedureKind,
    ResolverInput, ResolverOutput,
};

pub(super) type InvokeFn<TCtx> =
    Box<dyn Fn(TCtx, &mut dyn InputValueInner) -> Result<ProcedureStream, InternalError>>;

/// Represents a single operations on the server that can be executed.
///
/// A [`Procedure`] is built from a [`ProcedureBuilder`] and holds the type information along with the logic to execute the operation.
///
pub struct Procedure<TCtx = ()> {
    pub(super) ty: ProcedureKind,
    pub(super) input: fn(&mut TypeDefs) -> DataType,
    pub(super) result: fn(&mut TypeDefs) -> DataType,
    pub(super) handler: InvokeFn<TCtx>,
}

impl<TCtx> fmt::Debug for Procedure<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure").finish()
    }
}

impl<TCtx> Procedure<TCtx>
where
    TCtx: 'static,
{
    /// Construct a new procedure using [`ProcedureBuilder`].
    pub fn builder<I, R, TError>() -> ProcedureBuilder<TError, TCtx, TCtx, I, R>
    where
        TError: error::Error + Send + 'static,
        // Only the first layer (middleware or the procedure) needs to be a valid input/output type
        I: ResolverInput,
        R: ResolverOutput<TError>,
    {
        ProcedureBuilder {
            build: Box::new(|meta, _, handler| {
                // TODO: Don't be `Arc<Box<_>>` just `Arc<_>`
                let handler = Arc::new(handler);

                Procedure {
                    ty: meta.kind(),
                    input: |type_map| I::data_type(type_map),
                    result: |type_map| R::data_type(type_map),
                    handler: Box::new(move |ctx, input| {
                        let fut = handler(
                            ctx,
                            I::from_value(ProcedureExecInput::new(input))?,
                            meta.clone(),
                        );

                        Ok(R::into_procedure_stream(fut.into_stream()))
                    }),
                }
            }),
        }
    }

    /// Export the [Specta](https://docs.rs/specta) types for this procedure.
    ///
    /// TODO - Use this with `rspc::typescript`
    ///
    /// # Usage
    /// ```rust
    /// todo!(); # TODO: Example
    /// ```
    pub fn types(
        &self,
        key: Cow<'static, str>,
        type_map: &mut TypeDefs,
    ) -> ProcedureTypeDefinition {
        ProcedureTypeDefinition {
            key,
            kind: self.ty,
            input: (self.input)(type_map),
            result: (self.result)(type_map),
        }
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
    pub fn exec<'de, T: ProcedureInput<'de>>(
        &self,
        ctx: TCtx,
        input: T,
    ) -> Result<ProcedureStream, InternalError> {
        match input.into_deserializer() {
            Ok(deserializer) => {
                let mut input = <dyn erased_serde::Deserializer>::erase(deserializer);
                (self.handler)(ctx, &mut input)
            }
            Err(input) => (self.handler)(ctx, &mut AnyInput(Some(input.into_value()))),
        }
    }
}

pub struct ProcedureTypeDefinition {
    pub key: Cow<'static, str>,
    pub kind: ProcedureKind,
    pub input: DataType,
    pub result: DataType,
}
