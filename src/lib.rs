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

mod compiled_router;
mod config;
mod error;
mod procedure_like;
mod router;
mod rspc;
mod selection;

pub use crate::rspc::*;
pub use compiled_router::*;
pub use config::*;
pub use error::*;
pub use procedure_like::*; // TODO: Move into procedure
pub use router::*;
pub use selection::*;

pub mod integrations;
pub mod internal;
