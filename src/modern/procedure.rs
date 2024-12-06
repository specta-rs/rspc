//! A procedure holds a single operation that can be executed by the server.
//!
//! A procedure is built up from:
//!  - any number of middleware
//!  - a single resolver function (of type `query`, `mutation` or `subscription`)
//!
//! Features:
//!  - Input types (Serde-compatible or custom)
//!  - Result types (Serde-compatible or custom)
//!  - [`Future`](#todo) or [`Stream`](#todo) results
//!  - Typesafe error handling
//!
//! TODO: Request flow overview
//! TODO: Explain, what a procedure is, return type/struct, middleware, execution order, etc
//!

mod builder;
mod meta;
mod procedure;
mod resolver_input;
mod resolver_output;

pub use builder::ProcedureBuilder;
pub use meta::{ProcedureKind, ProcedureMeta};
// pub use procedure::{Procedure, ProcedureTypeDefinition, UnbuiltProcedure};
pub use resolver_input::ResolverInput;
pub use resolver_output::ResolverOutput;
