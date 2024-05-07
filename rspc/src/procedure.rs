//! TODO: Explain, what a procedure is, return type/struct, middleware, execution order, etc

mod builder;
mod erased_fut;
mod output;
mod procedure;
mod result;

pub use builder::ProcedureBuilder;
pub use output::Output;
pub use procedure::Procedure;
pub use result::ProcedureResult;
