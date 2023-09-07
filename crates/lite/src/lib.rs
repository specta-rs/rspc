//! A playground for making rspc a bit more light.
//!
//! This will probs become `rspc-core` at some point. TODO: How to keep rspc and rspc-core's version in sync even with breaking changes?
//!
//! Limitations with this redesign:
//!  - MwMapper will not work for now -> They *can* do but *should* they?
//!

mod error;
mod executable;
mod request;
mod resolver;

pub use error::*;
pub use executable::*;
pub use request::*;
pub use resolver::*;
