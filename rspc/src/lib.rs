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

#[allow(unused)]
pub use languages::*;
pub use procedure_kind::ProcedureKind;
pub use router::Router2;
pub use types::Types;

// TODO: These will come in the future.
#[cfg(not(feature = "unstable"))]
pub(crate) use modern::State;
#[cfg(not(feature = "unstable"))]
pub(crate) use procedure::Procedure2;

#[cfg(feature = "unstable")]
pub use as_date::AsDate;
#[cfg(feature = "unstable")]
pub use modern::{
    middleware, procedure::ProcedureBuilder, procedure::ProcedureMeta, procedure::ResolverInput,
    procedure::ResolverOutput, Error as Error2, Extension, Stream,
};
#[cfg(feature = "unstable")]
pub use procedure::Procedure2;

pub use rspc_core::{
    DeserializeError, DowncastError, DynInput, Procedure, ProcedureError, ProcedureStream,
    ProcedureStreamMap, ProcedureStreamValue, Procedures, ResolverError, State,
};

// Legacy stuff
#[cfg(not(feature = "nolegacy"))]
mod legacy;

#[cfg(not(feature = "nolegacy"))]
pub(crate) use legacy::interop;

// These remain to respect semver but will all go with the next major.
#[allow(deprecated)]
#[cfg(not(feature = "nolegacy"))]
pub use legacy::{
    internal, test_result_type, test_result_value, typedef, Config, DoubleArgMarker,
    DoubleArgStreamMarker, Error, ErrorCode, ExecError, ExecKind, ExportError, FutureMarker,
    Middleware, MiddlewareBuilder, MiddlewareContext, MiddlewareLike,
    MiddlewareWithResponseHandler, RequestLayer, Resolver, ResultMarker, Router, RouterBuilder,
    SerializeMarker, StreamResolver,
};
#[cfg(not(feature = "nolegacy"))]
pub use rspc_core::LegacyErrorInterop;
