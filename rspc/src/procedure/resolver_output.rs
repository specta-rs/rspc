use std::{convert::Infallible, error, marker::PhantomData};

use futures::{Stream, StreamExt};
use serde::Serialize;
use specta::{DataType, DefOpts, EnumVariant, Type, TypeDefs};

use super::{ProcedureOutput, ProcedureStream};

/// A type which can be returned from a procedure.
///
/// This has a default implementation for all [`Serialize`](serde::Serialize) types.
///
/// ## How this works?
///
/// We call [`Self::into_procedure_stream`] with the stream produced by the users handler and it will produce the [`ProcedureStream`] which is returned from the [`Procedure::exec`](super::Procedure::exec) call. If the user's handler was a [`Future`](std::future::Future) it will be converted into a [`Stream`](futures::Stream) by rspc.
///
/// For each value the [`Self::into_procedure_stream`] implementation **must** defer to [`Self::into_procedure_result`] to convert the value into a [`ProcedureOutput`]. rspc provides a default implementation that takes care of this for you so don't override it unless you have a good reason.
///
/// ## Implementation for custom types
/// ```rust
/// pub struct MyCoolThing(pub String);
///
/// impl<TErr: std::error::Error> ResolverOutput<Self, TErr> for MyCoolThing {
///     fn into_procedure_result(self) -> Result<ProcedureOutput, TErr> {
///        Ok(todo!()) // Refer to ProcedureOutput's docs
///     }
/// }
///
/// fn usage_within_rspc() {
///     <Procedure>::builder().query(|_, _: ()| async move { MyCoolThing("Hello, World!".to_string()) });
/// }
/// ```
pub trait ResolverOutput<M, TErr: error::Error>: Sized {
    /// Convert the procedure and any async part of the value into a [`ProcedureStream`].
    ///
    /// This primarily exists so the [`rspc::Stream`](crate::Stream) implementation can merge it's stream into the procedure stream.
    fn into_procedure_stream(
        procedure: impl Stream<Item = Self> + Send + 'static,
    ) -> ProcedureStream<TErr> {
        ProcedureStream::from_stream(procedure.map(|v| v.into_procedure_result()))
    }

    fn data_type(type_map: &mut TypeDefs) -> DataType;

    /// Convert the value from the user into a [`ProcedureOutput`].
    fn into_procedure_result(self) -> Result<ProcedureOutput, TErr>;
}

impl<T, TErr> ResolverOutput<Self, TErr> for T
where
    T: Serialize + Type + Send + 'static,
    TErr: error::Error,
{
    fn data_type(type_map: &mut TypeDefs) -> DataType {
        T::definition(DefOpts {
            parent_inline: false,
            type_map,
        })
        .unwrap() // Specta v2 doesn't panic
    }

    fn into_procedure_result(self) -> Result<ProcedureOutput, TErr> {
        Ok(ProcedureOutput::with_serde(self))
    }
}

pub struct ResultMarker<M>(Infallible, PhantomData<M>);
impl<T, M, TErr> ResolverOutput<ResultMarker<M>, TErr> for Result<T, TErr>
where
    T: ResolverOutput<M, TErr>,
    TErr: error::Error,
{
    fn data_type(type_map: &mut TypeDefs) -> DataType {
        // TODO: Should we wrap into a `Result`

        // export type Result<TOk, TErr> =
        // | { status: "ok"; data: TOk }
        // | { status: "error"; error: TErr };

        // DataType::Enum(specta::EnumType::Untagged {
        //     variants: vec![EnumVariant::Unnamed((T::data_type(type_map)))],
        //     generics: vec![],
        // })

        todo!();
    }

    fn into_procedure_result(self) -> Result<ProcedureOutput, TErr> {
        self?.into_procedure_result()
    }
}

pub struct StreamMarker<M>(Infallible, PhantomData<M>);
impl<S, M, TErr> ResolverOutput<StreamMarker<M>, TErr> for crate::Stream<S>
where
    S: Stream + Send + 'static,
    S::Item: ResolverOutput<M, TErr>,
    TErr: error::Error,
{
    fn data_type(type_map: &mut TypeDefs) -> DataType {
        S::Item::data_type(type_map)
    }

    fn into_procedure_stream(
        procedure: impl Stream<Item = Self> + Send + 'static,
    ) -> ProcedureStream<TErr> {
        ProcedureStream::from_stream(
            procedure
                .map(|v| v.0)
                .flatten()
                .map(|v| v.into_procedure_result()),
        )
    }

    fn into_procedure_result(self) -> Result<ProcedureOutput, TErr> {
        panic!("returning nested rspc::Stream's is not currently supported.")
    }
}
