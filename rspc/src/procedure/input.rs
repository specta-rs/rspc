use std::any::Any;

use serde::de::DeserializeOwned;

use super::InputValue;

pub trait Input: Sized + 'static {
    type Input: DeserializeOwned;

    fn deserialize(self) -> Option<Self::Input>;

    fn from_value(value: InputValue) -> Option<Self>;
}

impl<T: DeserializeOwned + Any + 'static> Input for T {
    type Input = Self;

    fn deserialize(self) -> Option<Self::Input> {
        Some(self)
    }

    fn from_value(value: InputValue) -> Option<Self> {
        value.deserialize().ok() // TODO: Error handling
    }
}
