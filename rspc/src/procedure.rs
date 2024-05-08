//! TODO: Explain, what a procedure is, return type/struct, middleware, execution order, etc

mod r#async;
mod builder;
mod input;
mod output;
mod procedure;
mod result;

pub use builder::ProcedureBuilder;
pub use input::{AnyInput, Input};
pub use output::Output;
pub use procedure::Procedure;
pub use r#async::{ProcedureExecResult, ProcedureExecResultFuture, ProcedureExecResultStream};
pub use result::ProcedureResult;
