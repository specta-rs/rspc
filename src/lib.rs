mod config;
mod error;
mod handle_request;
mod middleware;
mod procedure;
mod resolver;
mod resolver_result;
mod router;
mod router_builder;
mod selection;
mod types;

pub use config::*;
pub use error::*;
pub use handle_request::*;
pub use middleware::*;
pub use procedure::*;
pub use resolver::*;
pub use resolver_result::*;
pub use router::*;
pub use router_builder::*;
pub use selection::*;
pub use types::*;

pub mod integrations;

pub use specta::RSPCType as Type;

#[cfg(debug_assertions)]
pub fn test_result_type<T: specta::Type + serde::Serialize>() {
    panic!("You should not call `test_type` at runtime. This is just a debugging tool.");
}

#[cfg(debug_assertions)]
pub fn test_result_value<T: specta::Type + serde::Serialize>(_t: T) {
    panic!("You should not call `test_type` at runtime. This is just a debugging tool.");
}

#[doc(hidden)]
pub mod internal {
    pub use specta;
}
