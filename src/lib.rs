//! rspc is a framework for building typesafe web backends in Rust.
//!
//! Powered by [Specta](https://docs.rs/specta)'s powerful language exporting, rspc comes with integrations for [Axum](https://docs.rs/axum) and [Tauri](https://docs.rs/tauri) out of the box. This project brings the next level developer experience inspired by [tRPC](https://trpc.io) to your Rust stack.
//!
//! ## WARNING
//!
//! Checkout the official docs at <https://rspc.dev>. This documentation is generally written **for authors of middleware and adapter**.
//!
// #![forbid(unsafe_code)] // TODO
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::panic_in_result_fn,
    // missing_docs
)] // TODO: Move to workspace lints
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

mod procedure;
mod procedure_kind;
mod router;

pub use procedure_kind::ProcedureKind2;
pub use router::Router2;

// TODO: These will come in the future.
pub(crate) use procedure::Procedure2;
pub(crate) type State = ();

// TODO: Expose everything from `rspc_core`?

// Legacy stuff
mod legacy;

// These remain to respect semver but will all go with the next major.
#[allow(deprecated)]
pub use legacy::{
    internal, test_result_type, test_result_value, typedef, Config, DoubleArgMarker,
    DoubleArgStreamMarker, Error, ErrorCode, ExecError, ExecKind, ExportError, FutureMarker,
    Middleware, MiddlewareBuilder, MiddlewareContext, MiddlewareLike,
    MiddlewareWithResponseHandler, RequestLayer, Resolver, ResultMarker, Router, RouterBuilder,
    SerializeMarker, StreamResolver,
};
