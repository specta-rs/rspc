// TODO: Seal this module

/// TODO
pub trait ContentType {}

/// TODO
pub struct Json;

impl ContentType for Json {}

/// TODO
pub trait ContentTypes {
    // type Result;
}

impl ContentTypes for () {}

impl<T: ContentType> ContentTypes for T {}

impl<A: ContentType, B: ContentType> ContentTypes for (A, B) {}
