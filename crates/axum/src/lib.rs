//! rspc-axum: Axum integration for [rspc](https://rspc.dev).
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

mod extractors;
mod jsonrpc;
mod jsonrpc_exec;
mod legacy;
mod v2;

pub use legacy::endpoint;
pub use v2::endpoint2;
