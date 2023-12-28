//! Integrate rspc with an [Axum](https://docs.rs/axum/latest/axum/) HTTP server so it can be accessed from your frontend.
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::panic_in_result_fn,
    // missing_docs
)]
#![forbid(unsafe_code)]
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use axum::Router;

pub fn endpoint() -> Router {
    Router::new()
    // TODO: .nest("/todo", router)
}
