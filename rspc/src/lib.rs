//! rspc is a framework for building typesafe web backends in Rust.
//!
//! Powered by [Specta](https://docs.rs/specta)'s powerful language exporting, rspc comes with integrations for [Axum](https://docs.rs/axum) and [Tauri](https://docs.rs/tauri) out of the box. This project brings the next level developer experience inspired by [tRPC](https://trpc.io) to your Rust stack.
//!
//! ## WARNING
//!
//! Checkout the official docs at <https://rspc.dev>. This documentation is generally written **for authors of middleware and adapter**.
//!
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

mod as_date;
mod languages;
pub(crate) mod modern;
mod procedure;
mod procedure_kind;
mod router;
mod types;
pub(crate) mod util;

#[cfg(feature = "legacy")]
#[cfg_attr(docsrs, doc(cfg(feature = "legacy")))]
pub mod legacy;

#[allow(unused)]
pub use languages::*;
pub use procedure_kind::ProcedureKind;
pub use router::Router2;
pub use types::Types;

pub use as_date::AsDate;
pub use modern::{
    middleware, procedure::ProcedureBuilder, procedure::ProcedureMeta, procedure::ResolverInput,
    procedure::ResolverOutput, Error as Error2, Extension, Stream,
};
pub use procedure::Procedure2;

pub use rspc_procedure::{
    flush, DeserializeError, DowncastError, DynInput, DynOutput, Procedure, ProcedureError,
    ProcedureStream, ProcedureStreamMap, Procedures, ResolverError, State,
};
