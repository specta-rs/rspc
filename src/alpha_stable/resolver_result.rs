use std::marker::PhantomData;

#[derive(Clone, Copy)]
pub enum RequestKind {
    Query,
    Mutation,
}

pub struct RequestLayerMarker<T>(RequestKind, PhantomData<T>);

impl<T> RequestLayerMarker<T> {
    pub fn new(kind: RequestKind) -> Self {
        Self(kind, Default::default())
    }

    pub fn kind(&self) -> RequestKind {
        self.0
    }
}

pub struct StreamLayerMarker<T>(PhantomData<T>);

impl<T> StreamLayerMarker<T> {
    pub fn new() -> Self {
        Self(Default::default())
    }
}
