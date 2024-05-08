use std::any::Any;

use serde::Serialize;

// TODO: This should be public but sealed????
pub trait Input: 'static {
    type T: Serialize;

    fn value(self) -> Option<Self::T>;
}

impl<T: Serialize + Any + 'static> Input for T {
    type T = T;

    fn value(self) -> Option<Self::T> {
        Some(self)
    }
}

pub struct AnyInput<T>(T);

impl<T: Serialize + Any + 'static> Input for AnyInput<T> {
    type T = ();

    fn value(self) -> Option<Self::T> {
        None
    }
}

/// Sealed methods and keep `serde_value` out of the public API.
pub(super) trait InputSealed: 'static {
    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self)
    }

    fn to_value(&self) -> Option<Result<serde_value::Value, serde_value::SerializerError>> {
        None
    }
}
impl<T: Input> InputSealed for T {}
