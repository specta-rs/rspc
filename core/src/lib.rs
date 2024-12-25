//! rspc-core: Core interface for [rspc](https://docs.rs/rspc).
//!
//! TODO: Describe all the types and why the split?
//! TODO: This is kinda like `tower::Service`
//! TODO: Why this crate doesn't depend on Specta.
//! TODO: Discuss the traits that need to be layered on for this to be useful.
//! TODO: Discuss how middleware don't exist here.
//!
//! TODO: Results must be `'static` because they have to escape the closure.
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png",
    html_favicon_url = "https://github.com/specta-rs/rspc/raw/main/.github/logo.png"
)]

mod dyn_input;
mod dyn_output;
mod error;
mod interop;
mod logger;
mod procedure;
mod procedures;
mod state;
mod stream;

pub use dyn_input::DynInput;
pub use dyn_output::DynOutput;
pub use error::{DeserializeError, DowncastError, ProcedureError, ResolverError};
#[doc(hidden)]
pub use interop::LegacyErrorInterop;
pub use procedure::Procedure;
pub use procedures::Procedures;
pub use state::State;
pub use stream::{flush, ProcedureStream, ProcedureStreamMap};
