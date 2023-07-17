//! Some work in progress API that are not typesafe.
//!
//! WARNING: This module does not follow semver so may change at any time and can also break rspc's typesafe guarantees if not used correctly.

#![allow(clippy::unwrap_used)] // TODO: Fix this

mod mw_arg_mapper;

pub use mw_arg_mapper::*;
