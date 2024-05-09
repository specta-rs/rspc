use std::any::Any;

use serde::Deserializer;

pub trait Argument: 'static {
    type Value: Any + 'static;

    fn into_value(self) -> Self::Value;
}
