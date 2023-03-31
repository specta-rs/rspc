//! This module contains Alpha APIs that are used **under the hood** by the legacy code.
//! It's important that breaking changes to these APIs don't reach userspace!
//!
//! WARNING: Anything in this module does not follow semantic versioning until it's released however the API is fairly stable at this point.
//!

mod resolver_function;
mod resolver_result;

pub use resolver_function::*;
pub use resolver_result::*;

// TODO: Move this all out of here into another file

use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub enum RequestKind {
    Query,
    Mutation,
}

// TODO: I don't wanna call these markers cause they are runtime not just type level. Rename them.

#[doc(hidden)]
pub struct RequestLayerMarker<T>(RequestKind, PhantomData<T>);

impl<T> RequestLayerMarker<T> {
    pub fn new(kind: RequestKind) -> Self {
        Self(kind, Default::default())
    }

    pub fn kind(&self) -> RequestKind {
        self.0
    }
}

#[doc(hidden)]
pub struct StreamLayerMarker<T>(PhantomData<T>);

impl<T> StreamLayerMarker<T> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}
