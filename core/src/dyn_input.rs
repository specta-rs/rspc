use std::{
    any::{type_name, Any},
    fmt,
};

use serde::{de::Error, Deserialize};

use crate::{DeserializeError, DowncastError, ProcedureError};

// It would be really nice if this with `&'a DynInput<'de>` but that would require `#[repr(transparent)]` with can only be constructed with unsafe which is probally not worth it.

/// TODO
pub struct DynInput<'a, 'de> {
    inner: DynInputInner<'a, 'de>,
    pub(crate) type_name: &'static str,
}

enum DynInputInner<'a, 'de> {
    Value(&'a mut (dyn Any + Send)),
    Deserializer(&'a mut (dyn erased_serde::Deserializer<'de> + Send)),
}

impl<'a, 'de> DynInput<'a, 'de> {
    pub fn new_value<T: Send + 'static>(value: &'a mut Option<T>) -> Self {
        Self {
            inner: DynInputInner::Value(value),
            type_name: type_name::<T>(),
        }
    }

    pub fn new_deserializer<D: erased_serde::Deserializer<'de> + Send>(
        deserializer: &'a mut D,
    ) -> Self {
        Self {
            inner: DynInputInner::Deserializer(deserializer),
            type_name: type_name::<D>(),
        }
    }

    /// TODO
    pub fn deserialize<T: Deserialize<'de>>(self) -> Result<T, ProcedureError> {
        let DynInputInner::Deserializer(deserializer) = self.inner else {
            return Err(ProcedureError::Deserialize(DeserializeError(
                erased_serde::Error::custom(format!(
                    "attempted to deserialize from value '{}' but expected deserializer",
                    self.type_name
                )),
            )));
        };

        erased_serde::deserialize(deserializer)
            .map_err(|err| ProcedureError::Deserialize(DeserializeError(err)))
    }

    /// TODO
    pub fn value<T: 'static>(self) -> Result<T, DowncastError> {
        let DynInputInner::Value(value) = self.inner else {
            return Err(DowncastError {
                from: None,
                to: type_name::<T>(),
            });
        };
        Ok(value
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
