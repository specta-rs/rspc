//! rspc: A blazingly fast and easy to use TRPC-like server for Rust.
//!
//! Checkout the official docs <https://rspc.dev>
//!
#![forbid(unsafe_code)]
// #![allow(warnings)] // TODO: Remove this
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

mod config;
mod error;
mod layer;
mod middleware;
mod old_router;
mod procedure;
mod procedure_like;
mod router;
mod router_builder;
mod router_builder_like;
mod rspc;
mod selection;

pub use config::*;
pub use error::*;
pub use layer::*;
pub use middleware::*;
pub use old_router::*;
pub use procedure::*;
pub use procedure_like::*;
pub use router::*;
pub use router_builder::*;
pub use router_builder_like::*;
pub use rspc::*;
pub use selection::*;

pub mod integrations;
pub mod internal;
