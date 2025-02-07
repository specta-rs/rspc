//! rspc is a framework for building typesafe web backends in Rust.
//!
//! Powered by [Specta](https://docs.rs/specta)'s powerful language exporting, rspc comes with integrations for [Axum](https://docs.rs/axum) and [Tauri](https://docs.rs/tauri) out of the box. This project brings the next level developer experience inspired by [tRPC](https://trpc.io) to your Rust stack.
//!
//! ## WARNING
//!
//! Checkout the official docs at <https://rspc.dev>. This documentation is generally written **for authors of middleware and adapter**.
//!
// #![forbid(unsafe_code)] // TODO
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

pub mod middleware;

mod as_date;
mod error;
mod extension;
mod languages;
mod procedure;
mod procedure_kind;
mod router;
mod stream;
mod types;
pub(crate) mod util;

#[cfg(feature = "legacy")]
#[cfg_attr(docsrs, doc(cfg(feature = "legacy")))]
pub mod legacy;

#[cfg(not(feature = "legacy"))]
pub(crate) mod legacy;

pub use as_date::AsDate;
pub use error::Error;
pub use extension::Extension;
#[allow(unused)]
pub use languages::*;
pub use procedure::{
    ErasedProcedure, Procedure, ProcedureBuilder, ProcedureMeta, ResolverInput, ResolverOutput,
};
pub use procedure_kind::ProcedureKind;
pub use router::Router;
pub use stream::Stream;
pub use types::Types;

// We only re-export types that are useful for a general user.
pub use rspc_procedure::{
    flush, DynInput, ProcedureError, ProcedureStream, Procedures, ResolverError, State,
};

// TODO: Potentially remove these once Axum stuff is sorted.
pub use rspc_procedure::{DynOutput, ProcedureStreamMap};
