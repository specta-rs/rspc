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

pub(crate) mod interop;
mod languages;
mod procedure;
mod procedure_kind;
mod router;
mod types;

#[allow(unused)]
pub use languages::*;
pub use procedure_kind::ProcedureKind;
pub use router::Router2;
pub use types::Types;

#[deprecated = "This stuff is unstable. Don't use it unless you know what your doing"]
pub mod modern;

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
