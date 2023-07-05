//! rspc: A blazingly fast and easy to use TRPC-like server for Rust.
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
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub mod unstable;

mod compiled_router;
mod error;
mod router;
mod router_error;
mod rspc;
mod selection;

pub use crate::rspc::*;
pub use compiled_router::*;
pub use error::*;
pub use router::*;
pub use router_error::*;
pub use selection::*;

pub mod integrations;
pub mod internal;
