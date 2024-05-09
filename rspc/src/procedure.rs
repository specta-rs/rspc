//! TODO: Explain, what a procedure is, return type/struct, middleware, execution order, etc
//!
//!
//!
//!
//! Area's that need more work:
//!  - `Procedure::exec` and `Procedure::exec_any` should be merged into one
//!  - handling of result types is less efficient that it could be

mod argument;
mod builder;
mod input;
mod input_value;
mod output;
mod output_value;
mod procedure;
mod stream;

pub use argument::Argument;
pub use builder::ProcedureBuilder;
pub use input::Input;
pub use input_value::InputValue;
pub use output::Output;
pub use output_value::ProcedureResult;
pub use procedure::Procedure;
pub use stream::ProcedureStream;

// TODO: Remove this, it's just as a prototype
use std::pin::Pin;
#[doc(hidden)]
pub struct File<T = Pin<Box<dyn tokio::io::AsyncWrite>>>(pub T);
impl<T: tokio::io::AsyncWrite + 'static> Output for File<T> {
    fn into_result(self) -> ProcedureResult {
        let result: File = File(Box::pin(self.0));
        ProcedureResult::new(result)
    }
}
impl<F: tokio::io::AsyncWrite + 'static> Argument for File<F> {
    type Value = File;

    fn into_value(self) -> Self::Value {
        // TODO: Only reallocate if not already `Pin<Box<_>>`
        File(Box::pin(self.0))
    }
}
impl Input for File {
    fn from_value(value: InputValue<Self>) -> Result<Self, ()> {
        Ok(value.downcast().ok_or(())?)
    }
}
