use std::any::Any;

use serde::de::DeserializeOwned;

use super::ProcedureInput;

pub trait Input: Sized + Any + 'static {
    fn from_value(value: ProcedureInput<Self>) -> Result<Self, ()>;
}

impl<T: DeserializeOwned + 'static> Input for T {
    fn from_value(value: ProcedureInput<Self>) -> Result<Self, ()> {
        Ok(value.deserialize()?)
    }
}
