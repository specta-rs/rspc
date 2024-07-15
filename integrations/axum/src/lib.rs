//! rspc-axum: Axum integration for [rspc](https://rspc.dev).
#![cfg_attr(docsrs2, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png",
    html_favicon_url = "https://github.com/oscartbeaumont/rspc/raw/main/docs/public/logo.png"
)]

mod endpoint;
mod extractors;
mod file;

pub use endpoint::Endpoint;
pub use file::File;
