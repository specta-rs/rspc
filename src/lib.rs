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
// #![forbid(unsafe_code)]
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod error;
mod export_config;
pub mod internal;
pub mod procedure;
mod router;
pub mod router_builder;
pub mod router_builder2;
mod rspc;
pub mod types;

pub use export_config::ExportConfig;
pub use router::Router;
pub use rspc::Rspc;

// Avoid the user needing to depend on `rspc_core` and deal with keeping the versions in sync.
pub use rspc_core::Executor;
