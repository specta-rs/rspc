//! rspc core

// TODO: Clippy lints

mod executor;
mod format;
#[doc(hidden)]
pub mod internal;
mod router;
mod serializer;
mod task;

pub use executor::{Executor, Procedure};
pub use format::{Format, TODOSerializer};
pub use router::Router;
pub use serializer::Serializer;
pub use task::Task;
