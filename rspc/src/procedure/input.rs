use std::any::Any;

use serde::de::DeserializeOwned;

use super::InputValue;

pub trait Input: Sized + 'static {
    type Value: Any + 'static;

    fn into_value(self) -> Self::Value;

    fn from_value(value: InputValue) -> Result<Self, ()>;
}

impl<T: DeserializeOwned + 'static> Input for T {
    type Value = T;

    fn into_value(self) -> Self::Value {
        self
    }

    fn from_value(value: InputValue) -> Result<Self, ()> {
        value.deserialize().map_err(|_| ())
    }
}
