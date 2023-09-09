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
#![allow(clippy::module_inception)]
#![allow(clippy::type_complexity)] // TODO: Fix this and disable it
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub mod unstable;

mod blob;
mod built_router;
mod error;
mod router;
mod rspc;

pub use crate::rspc::*;
pub use built_router::*;
pub use error::*;
pub use router::*;

pub mod integrations;
pub mod internal;

#[cfg(feature = "unstable")]
#[cfg_attr(docsrs, doc(cfg(feature = "unstable")))]
pub use blob::Blob;

#[cfg(not(feature = "unstable"))]
pub(crate) use blob::Blob;
