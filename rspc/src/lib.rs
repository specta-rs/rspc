//! rspc is a framework for building typesafe web backends in Rust.
//!
//! Powered by [Specta](https://docs.rs/specta)'s powerful language exporting, rspc comes with integrations for [Axum](https://docs.rs/axum) and [Tauri](https://docs.rs/tauri) out of the box. This project brings the next level developer experience inspired by [tRPC](https://trpc.io) to your Rust stack.
//!
//! ## WARNING
//!
//! Checkout the official docs at <https://rspc.dev>. This documentation is generally written **for authors of middleware and adapter**.
//!
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod middleware;
pub mod procedure;

mod error;
mod infallible;
mod router;
mod state;
mod stream;

pub use error::Error;
pub use infallible::Infallible;
pub use router::{BuiltRouter, Router};
pub use state::State;
pub use stream::Stream;

#[doc(hidden)]
pub mod internal {
    // To make versioning easier we reexport it so libraries such as `rspc_axum` don't need a direct dependency on `specta`.
    pub use serde::Serialize;
    pub use specta::{DataType, Type, TypeMap}; // TODO: Why does rspc_axum care again?
}
