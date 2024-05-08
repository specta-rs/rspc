use std::any::{Any, TypeId};

use serde::{de::DeserializeOwned, Serialize, Serializer};
use serde_value::DeserializerError;

// TODO: This should be public but sealed????
pub trait Input {
    fn to_box_any(self: Box<Self>) -> Box<dyn Any>;

    fn to_value(&self) -> Option<Result<serde_value::Value, serde_value::SerializerError>> {
        None
    }
}

impl<T: Serialize + Any + 'static> Input for T {
    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self)
    }

    // TODO: This should shadow the `Input::to_value` method to achieve specialisation with stable Rust
    fn to_value(&self) -> Option<Result<serde_value::Value, serde_value::SerializerError>> {
        Some(serde_value::to_value(&self))
    }
}

pub struct AnyInput<T>(T);

impl<T: Any + 'static> Input for AnyInput<T> {
    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self)
    }
}
