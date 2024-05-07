//! TODO: Explain, what a procedure is, return type/struct, middleware, execution order, etc

mod r#async;
mod builder;
mod output;
mod procedure;
mod result;

pub use builder::ProcedureBuilder;
pub use output::Output;
pub use procedure::Procedure;
pub use r#async::{ProcedureExecResult, ProcedureExecResultFuture, ProcedureExecResultStream};
