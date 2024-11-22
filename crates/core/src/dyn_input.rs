use std::{any::Any, fmt};

use serde::{de::Error, Deserialize};

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
    pub fn value<T: 'static>(self) -> Option<T> {
        Some(
            self.value?
                .downcast_mut::<Option<T>>()?
                .take()
                // This takes method takes `self` and it's not `Clone` so it's not possible to double take the value.
                .expect("unreachable"),
        )
    }
}

impl<'a, 'de> fmt::Debug for DynInput<'a, 'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}

// TODO: Here or `error.rs`???
// TODO: impl Debug, Display, Error
#[derive(Debug)] // TODO: Remove
pub struct DeserializeError(pub(crate) erased_serde::Error);

// TODO: This should be convertable to a `ResolverError`.
