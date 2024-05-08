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

/// This is the internal version of `Input`.
///
/// It's sealed because:
///  - `serde_value` should not appear in the public API.
///  - It's methods are relatively unsafe due to the usage of `Option` to ensure dyn-safety with owned values.
pub(super) trait InputSealed: 'static {
    // This method returns `Option<T>` as `dyn Any` so we can take the value out of the `Option` while remaining dyn-safe.
    fn to_option_dyn_any(&mut self) -> &mut dyn Any;

    fn to_value(&mut self) -> Option<Result<serde_value::Value, serde_value::SerializerError>>;
}
impl<T: Input> InputSealed for Option<T> {
    fn to_option_dyn_any(&mut self) -> &mut dyn Any {
        self
    }

    fn to_value(&mut self) -> Option<Result<serde_value::Value, serde_value::SerializerError>> {
        self.take()
            .expect("value already taken")
            .value()
            .map(serde_value::to_value)
    }
}
