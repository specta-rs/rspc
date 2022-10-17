//! normi: a normalised caching system in Rust. Designed to work with rspc.
#![forbid(unsafe_code)]
#![warn(
    clippy::all,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::panic,
    clippy::todo,
    clippy::panic_in_result_fn,
    // missing_docs
)]

mod object;
#[cfg(feature = "rspc")]
mod rspc;

#[cfg(feature = "rspc")]
pub use crate::rspc::*;
pub use normi_macros::*;
pub use object::*;

// plz rename types in this module so they are gonna show up in rust-analyzer recommended imports for external crates
pub mod internal {
    pub use serde::Serialize;
    pub use serde_json::{to_value, Error, Value};
    pub use specta::Type;
}
