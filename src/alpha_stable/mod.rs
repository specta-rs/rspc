//! This module contains Alpha APIs that are used **under the hood** by the legacy code.
//! It's important that breaking changes to these APIs don't reach userspace!

mod resolver_function;
mod resolver_result;

pub use resolver_function::*;
pub use resolver_result::*;
