//! A resolver is the handler function defined by the user. A procedure is made up of a resolver and a type (query, mutation or subscription).

mod function;
mod layer;
mod result;

pub use function::*;
pub use layer::*;
pub use result::*;
