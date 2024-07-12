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
mod error;
mod exec_input;
mod input;
mod meta;
mod output;
mod procedure;
mod resolver_input;
mod resolver_output;
mod stream;

pub use builder::ProcedureBuilder;
pub use error::InternalError;
pub use exec_input::ProcedureExecInput;
pub use input::ProcedureInput;
pub use meta::ProcedureMeta;
pub use output::{ProcedureOutput, ProcedureOutputSerializeError};
pub use procedure::{Procedure, ProcedureType, ProcedureTypeDefinition};
pub use resolver_input::ResolverInput;
pub use resolver_output::ResolverOutput;
pub use stream::ProcedureStream;
