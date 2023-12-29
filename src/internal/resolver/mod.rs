//! A resolver is the handler function defined by the user. A procedure is made up of a resolver and a type (query, mutation or subscription).

mod function;
mod into_response;
mod layer;

pub(crate) use function::*;
pub(crate) use into_response::*;
pub(crate) use layer::*;
