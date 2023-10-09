//! rspc: A blazingly fast and easy to use tRPC-like server for Rust.
//!
//! Checkout the official docs <https://rspc.dev>
//!
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::panic_in_result_fn,
    // missing_docs
)]
#![forbid(unsafe_code)]
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod router_builder;
mod rspc;

pub use crate::rspc::*;
pub use router_builder::*;
pub use rspc_core::internal::router::*;

pub mod internal;

// TODO: Only reexport certain types
pub use rspc_core::error::*;
