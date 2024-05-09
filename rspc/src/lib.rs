//! rspc is a framework for building typesafe web backends in Rust.
//!
//! Powered by [Specta](https://docs.rs/specta)'s powerful language exporting, rspc comes with integrations for [Axum](https://docs.rs/axum) and [Tauri](https://docs.rs/tauri) out of the box. This project brings the next level DX inspired by [tRPC](https://trpc.io) to your Rust stack.
//!
//! ## WARNING
//!
//! Checkout the official docs at <https://rspc.dev>. This documentation is **for authors of middleware and adapter**,
//!
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::panic_in_result_fn,
    missing_docs
)]
#![forbid(unsafe_code)]
#![allow(clippy::module_inception)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod procedure;

pub struct Stream<T>(pub T);
