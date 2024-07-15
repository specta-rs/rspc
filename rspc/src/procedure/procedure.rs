use std::{borrow::Cow, error, fmt, sync::Arc};

use futures::FutureExt;
use specta::{DataType, TypeMap};

use crate::State;

use super::{
    exec_input::{AnyInput, InputValueInner},
    stream::ProcedureStream,
    InternalError, ProcedureBuilder, ProcedureExecInput, ProcedureInput, ProcedureKind,
    ProcedureMeta, ResolverInput, ResolverOutput,
};

pub(super) type InvokeFn<TCtx> = Arc<
    dyn Fn(TCtx, &mut dyn InputValueInner) -> Result<ProcedureStream, InternalError> + Send + Sync,
>;

/// Represents a single operations on the server that can be executed.
///
/// A [`Procedure`] is built from a [`ProcedureBuilder`] and holds the type information along with the logic to execute the operation.
///
pub struct Procedure<TCtx = ()> {
    kind: ProcedureKind,
    ty: ProcedureTypeDefinition,
    handler: InvokeFn<TCtx>,
}

impl<TCtx> Clone for Procedure<TCtx> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            ty: self.ty.clone(),
            handler: self.handler.clone(),
        }
    }
}

impl<TCtx> fmt::Debug for Procedure<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Procedure")
            .field("kind", &self.kind)
            .field("ty", &self.ty)
            .field("handler", &"...")
            .finish()
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
            build: Box::new(|kind, setups, handler| {
                // TODO: Don't be `Arc<Box<_>>` just `Arc<_>`
                let handler = Arc::new(handler);

                UnbuiltProcedure::new(move |key, state, type_map| {
                    let meta = ProcedureMeta::new(key.clone(), kind);
                    for setup in setups {
                        setup(state, meta.clone());
                    }

                    Procedure {
                        kind,
                        ty: ProcedureTypeDefinition {
                            key,
                            kind,
                            input: I::data_type(type_map),
                            result: R::data_type(type_map),
                        },
                        handler: Arc::new(move |ctx, input| {
                            let fut = handler(
                                ctx,
                                I::from_value(ProcedureExecInput::new(input))?,
                                meta.clone(),
                            );

                            Ok(R::into_procedure_stream(fut.into_stream()))
                        }),
                    }
                })
            }),
        }
    }
}

impl<TCtx> Procedure<TCtx> {
    pub fn kind(&self) -> ProcedureKind {
        self.kind
    }

    /// Export the [Specta](https://docs.rs/specta) types for this procedure.
    ///
    /// TODO - Use this with `rspc::typescript`
    ///
    /// # Usage
    ///
    /// ```rust
    /// todo!(); # TODO: Example
    /// ```
    pub fn types(&self) -> &ProcedureTypeDefinition {
        &self.ty
    }

    /// Execute a procedure with the given context and input.
    ///
    /// This will return a [`ProcedureStream`] which can be used to stream the result of the procedure.
    ///
    /// # Usage
    ///
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

#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureTypeDefinition {
    pub key: Cow<'static, str>,
    pub kind: ProcedureKind,
    pub input: DataType,
    pub result: DataType,
}

pub struct UnbuiltProcedure<TCtx>(
    Box<dyn FnOnce(Cow<'static, str>, &mut State, &mut TypeMap) -> Procedure<TCtx>>,
);

impl<TCtx> fmt::Debug for UnbuiltProcedure<TCtx> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnbuiltProcedure").finish()
    }
}

impl<TCtx> UnbuiltProcedure<TCtx> {
    pub(crate) fn new(
        build_fn: impl FnOnce(Cow<'static, str>, &mut State, &mut TypeMap) -> Procedure<TCtx> + 'static,
    ) -> Self {
        Self(Box::new(build_fn))
    }

    /// Build the procedure invoking all the setup functions.
    ///
    /// Generally you will not need to call this directly as you can give a [ProcedureFactory] to the [RouterBuilder::procedure] and let it take care of the rest.
    pub fn build(
        self,
        key: Cow<'static, str>,
        state: &mut State,
        type_map: &mut TypeMap,
    ) -> Procedure<TCtx> {
        (self.0)(key, state, type_map)
    }
}
