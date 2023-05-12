use std::marker::PhantomData;

use super::jsonrpc::RequestKind;

// TODO: I don't wanna call these markers cause they are runtime not just type level. Rename them.

// TODO: Make `pub(crate)`
#[doc(hidden)]
pub struct RequestLayerMarker<T>(RequestKind, PhantomData<T>);

impl<T> RequestLayerMarker<T> {
    pub(crate) fn new(kind: RequestKind) -> Self {
        Self(kind, Default::default())
    }

    pub(crate) fn kind(&self) -> RequestKind {
        self.0
    }
}

// TODO: Make `pub(crate)`
#[doc(hidden)]
pub struct StreamLayerMarker<T>(PhantomData<T>);

impl<T> StreamLayerMarker<T> {
    pub(crate) fn new() -> Self {
        Self(Default::default())
    }
}
