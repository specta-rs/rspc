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
//! Some tradeoffs:
//!  - Procedure return types are way less flexible -> but better errors and less complexity
//!
//! Area's that need more work:
//!  - handling of result types is less efficient that it could be
//!  - `rspc::Stream` in `rspc::Stream` will panic
//!  - Am I happy with `Output::into_procedure_stream`?
//!
//! Area's that require Rust improvements:
//!  - Serde zero-copy deserialization. We need a way to express `where F: Fn(..., I<'_>), I<'a>: Input<'a>`.

mod argument;
mod builder;
mod input;
mod output;
mod procedure;
mod procedure_input;
mod procedure_output;
mod stream;

pub use argument::Argument;
pub use builder::ProcedureBuilder;
pub use input::Input;
pub use output::Output;
pub use procedure::Procedure;
pub use procedure_input::ProcedureInput;
pub use procedure_output::ProcedureOutput;
pub use stream::ProcedureStream;

// TODO: Remove this, it's just as a prototype
use std::pin::Pin;
#[doc(hidden)]
pub struct File<T = Pin<Box<dyn tokio::io::AsyncWrite>>>(pub T);
impl<T: tokio::io::AsyncWrite + 'static> Output for File<T> {
    fn into_procedure_result(self) -> ProcedureOutput {
        let result: File = File(Box::pin(self.0));
        ProcedureOutput::new(result)
    }
}
impl<'de, F: tokio::io::AsyncWrite + 'static> Argument<'de> for File<F> {
    type Value = File;

    fn into_value(self) -> Self::Value {
        // TODO: Only reallocate if not already `Pin<Box<_>>`
        File(Box::pin(self.0))
    }
}
impl Input for File {
    fn from_value(value: ProcedureInput<Self>) -> Result<Self, ()> {
        Ok(value.downcast().ok_or(())?)
    }
}
