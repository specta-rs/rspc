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

pub mod notes;
pub mod procedure;
mod router;
mod rspc;

pub use router::Router;
pub use rspc::Rspc;

/// Return a [`Stream`](futures::Stream) of values from a [`Procedure::query`](procedure::ProcedureBuilder::query) or [`Procedure::mutation`](procedure::ProcedureBuilder::mutation).
///
/// ## Why not a subscription?
///
/// A [`subscription`](procedure::ProcedureBuilder::subscription) must return a [`Stream`](futures::Stream) so it would be fair to question when you would use this.
///
/// A [`query`](procedure::ProcedureBuilder::query) or [`mutation`](procedure::ProcedureBuilder::mutation) produce a single result where a subscription produces many discrete values.
///
/// Using [`rspc::Stream`](Self) within a query or mutation will result in your procedure returning a collection (Eg. `Vec`) of [`Stream::Item`](futures::Stream) on the frontend.
///
/// This means it would be well suited for streaming the result of a computation or database query while a subscription would be well suited for a chat room.
///
/// ## Usage
/// **WARNING**: This example shows the low-level procedure API. You should refer to [`Rspc`](crate::Rspc) for the high-level API.
/// ```rust
/// use futures::stream::once;
///
/// <Procedure>::builder().query(|_, _: ()| async move { rspc::Stream(once(async move { 42 })) });
/// ```
///
pub struct Stream<T>(pub T);
