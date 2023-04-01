//! rspc: A blazingly fast and easy to use TRPC-like server for Rust.
//!
//! Checkout the official docs <https://rspc.dev>
//!
#![forbid(unsafe_code)]
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::panic_in_result_fn,
    // missing_docs
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "alpha")]
#[cfg_attr(docsrs, doc(cfg(feature = "alpha")))]
// #[deprecated = "Being removed in `v1.0.0`. This will be in the root of the crate."] // TODO
pub mod alpha;
// #[deprecated = "Being removed in `v1.0.0`. This will be in the root of the crate."] // TODO
pub(crate) mod alpha_stable;
mod config;
mod error;
mod middleware;
mod resolver_result;
mod router;
mod router_builder;
mod selection;

pub use config::*;
pub use error::*;
pub use middleware::*;
pub use resolver_result::*;
pub use router::*;
pub use router_builder::*;

pub use selection::*;

pub mod integrations;
pub mod internal;

// #[deprecated = "Being removed in `v1.0.0`. Import this directly from the 'specta' crate."] // TODO
pub use specta::RSPCType as Type;

#[cfg(debug_assertions)]
#[allow(clippy::panic)]
// #[deprecated = "Being removed in `v1.0.0`. You can copy this helper into your own codebase if you still need it."] // TODO
pub fn test_result_type<T: specta::Type + serde::Serialize>() {
    panic!("You should not call `test_type` at runtime. This is just a debugging tool.");
}

#[cfg(debug_assertions)]
#[allow(clippy::panic)]
// #[deprecated = "Being removed in `v1.0.0`. You can copy this helper into your own codebase if you still need it."] // TODO
pub fn test_result_value<T: specta::Type + serde::Serialize>(_: T) {
    panic!("You should not call `test_type` at runtime. This is just a debugging tool.");
}
