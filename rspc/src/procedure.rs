//! TODO: Explain, what a procedure is, return type/struct, middleware, execution order, etc

mod builder;
mod output;
mod procedure;
mod result;

pub use builder::ProcedureBuilder;
pub use output::Output;
pub use procedure::{Procedure, ProcedureExecResult, ProcedureExecResultFuture};
