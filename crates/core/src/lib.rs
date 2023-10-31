//! Core traits and types for [rspc](https://docs.rs/rspc)
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::panic_in_result_fn,
    // missing_docs
)]
#![warn(unsafe_code)]
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod body;
mod body2;
mod layer;
mod middleware;
mod procedure_store;
mod router;
mod router_builder;
mod util;

pub use body2::{Body, BytesBody, StreamBody, ValueOrBytes};

// TODO: Reduce API surface in this??
pub mod error;

// TODO: Reduce API surface in this??
pub mod exec;

pub use router_builder::BuildError;

pub use router::Router;

#[doc(hidden)]
pub mod internal {
    //! rspc core internals.
    //!
    //! WARNING: Anything in this module or it's submodules does not follow semantic versioning as it's considered an implementation detail.

    pub mod router {
        pub use super::super::router::*;
    }

    pub use super::error::private::IntoResolverError;
    pub use super::util::{PinnedOption, PinnedOptionProj};

    pub use super::layer::Layer;
    pub use super::procedure_store::{build, ProcedureDef, ProcedureTodo, ProceduresDef};

    pub use super::middleware::{
        new_mw_ctx, Executable2, MiddlewareContext, MwV2Result, ProcedureKind, RequestContext,
    };

    pub use super::router_builder::{
        edit_build_error_name, new_build_error, BuildError, BuildErrorCause, BuildResult,
        ProcedureMap,
    };
}
