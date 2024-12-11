//! rspc-core: Core interface for [rspc](https://docs.rs/rspc).
//!
//! TODO: Describe all the types and why the split?
//! TODO: This is kinda like `tower::Service`
//! TODO: Why this crate doesn't depend on Specta.
//! TODO: Discuss the traits that need to be layered on for this to be useful.
//! TODO: Discuss how middleware don't exist here.
//!
//! TODO: A fundamental flaw of our current architecture is that results must be `'static` (hence can't serialize in-place). This is hard to solve due to `async fn`'s internals being sealed.
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

mod dyn_input;
mod error;
mod interop;
mod procedure;
mod procedures;
mod stream;

pub use dyn_input::DynInput;
pub use error::{DeserializeError, DowncastError, ProcedureError, ResolverError};
pub use interop::LegacyErrorInterop;
pub use procedure::Procedure;
pub use procedures::Procedures;
pub use stream::ProcedureStream;
