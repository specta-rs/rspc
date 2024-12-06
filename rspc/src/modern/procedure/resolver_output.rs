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

use futures::Stream;
use rspc_core::ProcedureStream;
use serde::Serialize;
use specta::{datatype::DataType, Generics, Type, TypeCollection};

use crate::modern::Error;

// TODO: Maybe in `rspc_core`??

/// TODO: bring back any correct parts of the docs above
pub trait ResolverOutput<TError>: Sized + Send + 'static {
    // /// Convert the procedure and any async part of the value into a [`ProcedureStream`].
    // ///
    // /// This primarily exists so the [`rspc::Stream`](crate::Stream) implementation can merge it's stream into the procedure stream.
    // fn into_procedure_stream(
    //     procedure: impl Stream<Item = Result<Self, TError>> + Send + 'static,
    // ) -> ProcedureStream
    // where
    //     TError: Error,
    // {
    //     ProcedureStream::from_stream(procedure.map(|v| v?.into_procedure_result()))
    // }

    // /// Convert the value from the user into a [`ProcedureOutput`].
    // fn into_procedure_result(self) -> Result<ProcedureOutput, TError>;

    // TODO: Be an associated type instead so we can constrain later for better errors????
    fn data_type(types: &mut TypeCollection) -> DataType;

    fn into_procedure_stream(self) -> ProcedureStream;
}

// TODO: Should this be `Result`?
impl<T, TError> ResolverOutput<TError> for T
where
    T: Serialize + Type + Send + 'static,
    TError: Error,
{
    fn data_type(types: &mut TypeCollection) -> DataType {
        T::inline(types, Generics::Definition)
    }

    fn into_procedure_stream(self) -> ProcedureStream {
        ProcedureStream::from_value(Ok(self))
    }
}

impl<TErr, S, T> ResolverOutput<TErr> for crate::modern::Stream<S>
where
    TErr: Send,
    S: Stream<Item = Result<T, TErr>> + Send + 'static,
    T: ResolverOutput<TErr>,
{
    fn data_type(types: &mut TypeCollection) -> DataType {
        T::data_type(types) // TODO: Do we need to do anything special here so the frontend knows this is a stream?
    }

    fn into_procedure_stream(self) -> ProcedureStream {
        // ProcedureStream::from_value(Ok(self))

        // ProcedureStream::from_stream(
        //     self.0
        //         .map(|v| match v {
        //             Ok(s) => {
        //                 s.0.map(|v| v.and_then(|v| v.into_procedure_result()))
        //                     .right_stream()
        //             }
        //             Err(err) => once(async move { Err(err) }).left_stream(),
        //         })
        //         .flatten(),
        // )

        todo!();
    }

    // fn into_procedure_stream(
    //     procedure: impl Stream<Item = Result<Self, TErr>> + Send + 'static,
    // ) -> ProcedureStream
    // where
    //     TErr: Error,
    // {
    //     ProcedureStream::from_stream(
    //         procedure
    //             .map(|v| match v {
    //                 Ok(s) => {
    //                     s.0.map(|v| v.and_then(|v| v.into_procedure_result()))
    //                         .right_stream()
    //                 }
    //                 Err(err) => once(async move { Err(err) }).left_stream(),
    //             })
    //             .flatten(),
    //     )
    // }

    // fn into_procedure_result(self) -> Result<ProcedureOutput, TErr> {
    //     panic!("returning nested rspc::Stream's is not currently supported.")
    // }
}
