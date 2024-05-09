use std::any::Any;

pub trait Argument: 'static {
    type Value: Any + 'static;

    fn into_value(self) -> Self::Value;
}
