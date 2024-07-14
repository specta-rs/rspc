use std::pin::Pin;

use rspc::{
    internal::{DataType, TypeDefs},
    procedure::{
        InternalError, ProcedureExecInput, ProcedureInput, ProcedureOutput, ResolverInput,
        ResolverOutput,
    },
};
use tokio::io::AsyncWrite;

// TODO: Clone, Debug, etc
pub struct File<T = Pin<Box<dyn AsyncWrite + Send>>>(pub T);

impl<T: AsyncWrite + Send + 'static, TErr> ResolverOutput<TErr> for File<T> {
    fn data_type(type_map: &mut TypeDefs) -> DataType {
        DataType::Any // TODO
    }

    fn into_procedure_result(self) -> Result<ProcedureOutput, TErr> {
        let result: File = File(Box::pin(self.0));
        Ok(ProcedureOutput::new(result))
    }
}

impl<'de, F: AsyncWrite + Send + 'static> ProcedureInput<'de> for File<F> {
    type Value = File;

    fn into_value(self) -> Self::Value {
        // TODO: Only reallocate if not already `Pin<Box<_>>`
        File(Box::pin(self.0))
    }
}

impl ResolverInput for File {
    fn data_type(type_map: &mut TypeDefs) -> DataType {
        DataType::Any // TODO
    }

    fn from_value(value: ProcedureExecInput<Self>) -> Result<Self, InternalError> {
        value.downcast()
    }
}
