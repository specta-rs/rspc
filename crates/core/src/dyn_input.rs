use std::{
    any::{type_name, Any},
    fmt,
};

use serde::{de::Error, Deserialize};

use crate::{DeserializeError, DowncastError};

/// TODO
pub struct DynInput<'a, 'de> {
    pub(crate) value: Option<&'a mut dyn Any>,
    pub(crate) deserializer: Option<&'a mut dyn erased_serde::Deserializer<'de>>,
    pub(crate) type_name: &'static str,
}

impl<'a, 'de> DynInput<'a, 'de> {
    /// TODO
    pub fn deserialize<T: Deserialize<'de>>(self) -> Result<T, DeserializeError> {
        erased_serde::deserialize(self.deserializer.ok_or(DeserializeError(
            erased_serde::Error::custom(format!(
                "attempted to deserialize from value '{}' but expected deserializer",
                self.type_name
            )),
        ))?)
        .map_err(|err| DeserializeError(err))
    }

    /// TODO
    pub fn value<T: 'static>(self) -> Result<T, DowncastError> {
        Ok(self
            .value
            .ok_or(DowncastError {
                from: None,
                to: type_name::<T>(),
            })?
            .downcast_mut::<Option<T>>()
            .ok_or(DowncastError {
                from: Some(self.type_name),
                to: type_name::<T>(),
            })?
            .take()
            // This takes method takes `self` and it's not `Clone` so it's not possible to double take the value.
            .expect("unreachable"))
    }
}

impl<'a, 'de> fmt::Debug for DynInput<'a, 'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
