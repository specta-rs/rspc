//! A procedure is the base primitive of rspc. It represents ...
//!
//! TODO: Explain, what a procedure is, return type/struct, middleware, execution order, etc
//!
//!
//! Features:
//!  - Input types (Serde-compatible or custom)
//!  - Result types (Serde-compatible or custom)
//!  - [`Future`](#todo) or [`Stream`](#todo) results
//!  - Typesafe error handling
//!
//!
//! TODO: Request flow overview

mod builder;
mod exec_input;
mod input;
mod output;
mod procedure;
mod resolver_input;
mod resolver_output;
mod stream;

pub use builder::ProcedureBuilder;
pub use exec_input::ProcedureExecInput;
pub use input::ProcedureInput;
pub use output::ProcedureOutput;
pub use procedure::Procedure;
pub use resolver_input::ResolverInput;
pub use resolver_output::ResolverOutput;
pub use stream::ProcedureStream;

// TODO: Remove this, it's just as a prototype
use std::pin::Pin;
#[doc(hidden)]
pub struct File<T = Pin<Box<dyn tokio::io::AsyncWrite>>>(pub T);
impl<T: tokio::io::AsyncWrite + 'static> ResolverOutput for File<T> {
    fn into_procedure_result(self) -> ProcedureOutput {
        let result: File = File(Box::pin(self.0));
        ProcedureOutput::new(result)
    }
}
impl<'de, F: tokio::io::AsyncWrite + 'static> ProcedureInput<'de> for File<F> {
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
