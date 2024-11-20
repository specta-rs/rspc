//! rspc: A blazingly fast and easy to use TRPC-like server for Rust.
#![forbid(unsafe_code)]
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
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

mod config;
mod error;
mod middleware;
mod resolver;
mod resolver_result;
mod router;
mod router_builder;
mod selection;

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

#[deprecated = "Not going to be included in 0.4.0. The function is 5 lines so copy into your project!"]
#[cfg(debug_assertions)]
#[allow(clippy::panic)]
pub fn test_result_type<T: specta::Type + serde::Serialize>() {
    panic!("You should not call `test_type` at runtime. This is just a debugging tool.");
}

#[deprecated = "Not going to be included in 0.4.0. The function is 5 lines so copy into your project!"]
#[cfg(debug_assertions)]
#[allow(clippy::panic)]
pub fn test_result_value<T: specta::Type + serde::Serialize>(_: T) {
    panic!("You should not call `test_type` at runtime. This is just a debugging tool.");
}
