// /// A type which can be returned from a procedure.
// ///
// /// This has a default implementation for all [`Serialize`](serde::Serialize) types.
// ///
// /// ## How this works?
// ///
// /// We call [`Self::into_procedure_stream`] with the stream produced by the users handler and it will produce the [`ProcedureStream`] which is returned from the [`Procedure::exec`](super::Procedure::exec) call. If the user's handler was a [`Future`](std::future::Future) it will be converted into a [`Stream`](futures::Stream) by rspc.
// ///
// /// For each value the [`Self::into_procedure_stream`] implementation **must** defer to [`Self::into_procedure_result`] to convert the value into a [`ProcedureOutput`]. rspc provides a default implementation that takes care of this for you so don't override it unless you have a good reason.
// ///
// /// ## Implementation for custom types
// ///
// /// ```rust
// /// pub struct MyCoolThing(pub String);
// ///
// /// impl<TErr: std::error::Error> ResolverOutput<Self, TErr> for MyCoolThing {
// ///     fn into_procedure_result(self) -> Result<ProcedureOutput, TErr> {
// ///        Ok(todo!()) // Refer to ProcedureOutput's docs
// ///     }
// /// }
// ///
// /// fn usage_within_rspc() {
// ///     <Procedure>::builder().query(|_, _: ()| async move { MyCoolThing("Hello, World!".to_string()) });
// /// }
// /// ```
// // TODO: Do some testing and set this + add documentation link into it.
// // #[diagnostic::on_unimplemented(
// //     message = "Your procedure must return a type that implements `serde::Serialize + specta::Type + 'static`",
// //     note = "ResolverOutput requires a `T where T: serde::Serialize + specta::Type + 'static` to be returned from your procedure"
// // )]

use futures_util::{Stream, TryStreamExt};
use rspc_procedure::{ProcedureError, ProcedureStream};
use serde::Serialize;
use specta::{datatype::DataType, Generics, Type, TypeCollection};

use crate::Error;

// TODO: Maybe in `rspc_procedure`??

/// TODO: bring back any correct parts of the docs above
pub trait ResolverOutput<TError>: Sized + Send + 'static {
    type T;

    // TODO: Be an associated type instead so we can constrain later for better errors????
    fn data_type(types: &mut TypeCollection) -> DataType;

    /// Convert the procedure into a [`Stream`].
    fn into_stream(self) -> impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static;

    /// Convert the stream into a [`ProcedureStream`].
    fn into_procedure_stream(
        stream: impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static,
    ) -> ProcedureStream;
}

impl<T, E> ResolverOutput<E> for T
where
    T: Serialize + Type + Send + Sync + 'static,
    E: Error,
{
    type T = T;

    fn data_type(types: &mut TypeCollection) -> DataType {
        T::inline(types, Generics::Definition)
    }

    fn into_stream(self) -> impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static {
        futures_util::stream::once(async move { Ok(self) })
    }

    fn into_procedure_stream(
        stream: impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static,
    ) -> ProcedureStream {
        ProcedureStream::from_stream(stream)
    }
}

impl<TErr, S, T> ResolverOutput<TErr> for crate::Stream<S>
where
    TErr: Error,
    S: Stream<Item = Result<T, TErr>> + Send + 'static,
    T: ResolverOutput<TErr>,
    // Should prevent nesting `Stream`s
    T::T: Serialize + Send + Sync + 'static,
{
    type T = T::T;

    fn data_type(types: &mut TypeCollection) -> DataType {
        T::data_type(types) // TODO: Do we need to do anything special here so the frontend knows this is a stream?
    }

    fn into_stream(self) -> impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static {
        self.0
            .map_ok(|v| v.into_stream())
            .map_err(|err| err.into_procedure_error())
            .try_flatten()
    }

    fn into_procedure_stream(
        stream: impl Stream<Item = Result<Self::T, ProcedureError>> + Send + 'static,
    ) -> ProcedureStream {
        ProcedureStream::from_stream(stream)
    }
}
