//! rspc-axum: [Axum](https://docs.rs/axum) integration for [rspc](https://rspc.dev).
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

mod endpoint;
mod extractors;
mod jsonrpc;
mod jsonrpc_exec;
// mod legacy;
mod request;
mod v2;

pub use endpoint::Endpoint;
pub use request::AxumRequest;
pub use v2::endpoint;
