//! The rspc 0.3.1 syntax implemented on top of the 0.4.0 core.
//!
//! This allows incremental migration from the old syntax to the new syntax with the minimal breaking changes.
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true",
    html_favicon_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true"
)]

mod config;
mod error;
mod middleware;
mod resolver;
mod resolver_result;
mod router;
mod router_builder;
mod selection;

#[cfg_attr(
    feature = "deprecated",
    deprecated = "This is replaced by `rspc::Typescript`"
)]
pub use config::Config;
pub use error::{Error, ErrorCode, ExecError, ExportError};
pub use middleware::{
    Middleware, MiddlewareBuilder, MiddlewareContext, MiddlewareLike, MiddlewareWithResponseHandler,
};
pub use resolver::{typedef, DoubleArgMarker, DoubleArgStreamMarker, Resolver, StreamResolver};
pub use resolver_result::{FutureMarker, RequestLayer, ResultMarker, SerializeMarker};
pub use router::{ExecKind, Router};
pub use router_builder::RouterBuilder;

pub mod internal;

#[cfg_attr(
    feature = "deprecated",
    deprecated = "This is no longer going to included. You can copy it into your project if you need it."
)]
#[cfg(debug_assertions)]
#[allow(clippy::panic)]
pub fn test_result_type<T: specta::Type + serde::Serialize>() {
    panic!("You should not call `test_type` at runtime. This is just a debugging tool.");
}

#[cfg_attr(
    feature = "deprecated",
    deprecated = "This is no longer going to included. You can copy it into your project if you need it."
)]
#[cfg(debug_assertions)]
#[allow(clippy::panic)]
pub fn test_result_value<T: specta::Type + serde::Serialize>(_: T) {
    panic!("You should not call `test_type` at runtime. This is just a debugging tool.");
}
