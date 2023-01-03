//! A blazingly fast and easy to use TRPC-like server for Rust.
//!
//! [Official docs](https://rspc.dev)
//!
#![forbid(unsafe_code)]
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::panic_in_result_fn,
    // rustdoc::all,
    // missing_docs
    // missing_panics_doc
    // missing_debug_implementations
)]
#![allow(clippy::cargo_common_metadata)] // TODO: I enable this because it's doing a false positive on the `normi-macros` crate.

mod config;
mod error;
mod router;
mod router_builder;
mod selection;

pub use config::*;
pub use error::*;
pub use router::*;
pub use router_builder::*;

pub use selection::*;

pub mod integrations;
pub mod internal;
pub mod plugins;

pub use specta::RSPCType as Type;

#[cfg(debug_assertions)]
#[allow(clippy::panic)]
pub fn test_result_type<T: specta::Type + serde::Serialize>() {
    panic!("You should not call `test_type` at runtime. This is just a debugging tool.");
}

#[cfg(debug_assertions)]
#[allow(clippy::panic)]
pub fn test_result_value<T: specta::Type + serde::Serialize>(_: T) {
    panic!("You should not call `test_type` at runtime. This is just a debugging tool.");
}
