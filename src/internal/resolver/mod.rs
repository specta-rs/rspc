//! A resolver is the handler function defined by the user. A procedure is made up of a resolver and a type (query, mutation or subscription).

mod function;
mod into_response;
mod result;

pub(crate) use function::*;
pub(crate) use into_response::*;
#[allow(unused_imports)] // TODO: Fix this
pub(crate) use result::*;
