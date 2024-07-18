//! Expose your [rspc](https://rspc.dev) application as an HTTP and/or WebSocket API using [Axum](https://github.com/tokio-rs/axum).
//!
//! # Example
//!
//! To get started you can copy the following example and run it with `cargo run`.
//!
//! ```rust
//! use axum::{
//!     routing::get,
//!     Router,
//! };
//!
//! #[tokio::main]
//! async fn main() {
//!     let router = rspc::Router::new().build().unwrap();
//!
//!     let app = Router::new()
//!         .route("/", get(|| async { "Hello, World!" }))
//!         .nest(
//!             "/rspc",
//!             rspc_axum::Endpoint::new(router.clone(), || ()),
//!         )
//!
//!     // run our app with hyper, listening globally on port 3000
//!     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//!     axum::serve(listener, app).await.unwrap();
//! }
//! ```
//!
//! Note: You must enable the `ws` feature to use WebSockets.
//!
//! # Features
//!
//! You can enable any of the following features to enable additional functionality:
//!
//! - `ws`: Support for WebSockets.
//! - `file`: Support for serving files.
//!
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

mod ctx_fn;
mod endpoint;
#[cfg(feature = "file")]
#[cfg_attr(docsrs, doc(cfg(feature = "file")))]
mod file;

pub use endpoint::Endpoint;

#[cfg(feature = "file")]
#[cfg_attr(docsrs, doc(cfg(feature = "file")))]
pub use file::File;
