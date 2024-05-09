use futures::{Stream, StreamExt};
use serde::Serialize;
use specta::Type;

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
/// impl Output for MyCoolThing {
///     fn into_procedure_result(self) -> ProcedureOutput {
///        Ok(todo!()) // Refer to ProcedureOutput's docs
///     }
/// }
///
/// fn usage_within_rspc() {
///     <Procedure>::builder().query(|_, _: ()| async move { MyCoolThing("Hello, World!".to_string()) });
/// }
/// ```
pub trait Output: Sized {
    /// Convert the procedure and any async part of the value into a [`ProcedureStream`].
    ///
    /// This primarily exists so the [`rspc::Stream`](crate::Stream) implementation can merge it's stream into the procedure stream.
    fn into_procedure_stream(
        procedure: impl Stream<Item = Self> + Send + 'static,
    ) -> ProcedureStream {
        ProcedureStream::from_stream(procedure.map(|v| v.into_procedure_result()))
    }

    /// Convert the value from the user into a [`ProcedureOutput`].
    fn into_procedure_result(self) -> ProcedureOutput;
}

impl<T> Output for T
where
    T: Serialize + Type + Send + 'static,
{
    fn into_procedure_result(self) -> ProcedureOutput {
        ProcedureOutput::with_serde(self)
    }
}

impl<S> Output for crate::Stream<S>
where
    S: Stream + Send + 'static,
    S::Item: Output,
{
    fn into_procedure_stream(
        procedure: impl Stream<Item = Self> + Send + 'static,
    ) -> ProcedureStream {
        ProcedureStream::from_stream(
            procedure
                .map(|v| v.0)
                .flatten()
                .map(|v| v.into_procedure_result()),
        )
    }

    fn into_procedure_result(self) -> ProcedureOutput {
        panic!("returning nested rspc::Stream's is not currently supported.")
    }
}
