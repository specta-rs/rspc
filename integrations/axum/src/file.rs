use std::pin::Pin;

use rspc::procedure::{ProcedureExecInput, ProcedureInput, ResolverInput, ResolverOutput};
use tokio::io::AsyncWrite;

// TODO: Clone, Debug, etc
pub struct File<T = Pin<Box<dyn AsyncWrite>>>(pub T);

impl<T: AsyncWrite + 'static> ResolverOutput for File<T> {
    fn into_procedure_result(self) -> ProcedureOutput {
        let result: File = File(Box::pin(self.0));
        ProcedureOutput::new(result)
    }
}

impl<'de, F: AsyncWrite + 'static> ProcedureInput<'de> for File<F> {
    type Value = File;

    fn into_value(self) -> Self::Value {
        // TODO: Only reallocate if not already `Pin<Box<_>>`
        File(Box::pin(self.0))
    }
}

impl ResolverInput for File {
    fn from_value(value: ProcedureExecInput<Self>) -> Result<Self, ()> {
        Ok(value.downcast().ok_or(())?)
    }
}
