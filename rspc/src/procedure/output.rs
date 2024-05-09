use std::future::Future;

use futures::{FutureExt, Stream, StreamExt};
use serde::Serialize;
use specta::Type;

use super::{ProcedureOutput, ProcedureStream};

pub trait Output: Sized {
    fn into_procedure_stream(
        procedure: impl Stream<Item = Self> + Send + 'static,
    ) -> ProcedureStream {
        ProcedureStream::from_stream(procedure.map(|v| v.into_procedure_result()))
    }

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
        todo!() // TODO: This would be hit if you return an `rspc::Stream` from an `rspc::Stream`
    }
}
