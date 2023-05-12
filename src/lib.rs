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

#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub mod unstable;

mod built_router;
mod config;
mod error;
mod procedure;
mod procedure_like;
mod router;
mod rspc;
mod selection;

pub use built_router::*;
pub use config::*;
pub use error::*;
pub use procedure::*;
pub use procedure_like::*;
pub use router::*;
pub use rspc::*;
pub use selection::*;

pub mod integrations;
pub mod internal;
