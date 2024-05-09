use std::any::Any;

use serde::de::DeserializeOwned;

use super::InputValue;

pub trait Input: Sized + 'static {
    fn from_value(value: InputValue) -> Option<Self>;
}

impl<T: DeserializeOwned + Any + 'static> Input for T {
    fn from_value(value: InputValue) -> Option<Self> {
        value.deserialize().ok() // TODO: Error handling
    }
}
