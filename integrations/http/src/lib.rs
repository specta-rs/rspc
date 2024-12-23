//! rspc-http: Generic HTTP adapter for [rspc](https://rspc.dev).
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

// TODO: Working extractors w/ Axum and Actix-web
// TODO: Websockets
// TODO: Supporting non-json formats
// TODO: `File` type abstraction

// TODO: Custom cookies, headers, etc

mod execute;
mod file;
mod socket;

pub use execute::*;
