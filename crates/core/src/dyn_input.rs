use std::{any::Any, fmt};

use serde::Deserialize;

// TODO: Could we call this `DynValue` instead????

/// TODO
pub struct DynInput<'a, 'de> {
    // TODO: Or seal fields to this module and have constructors???
    pub(crate) value: Option<&'a mut dyn Any>,
    pub(crate) deserializer: Option<&'a mut dyn erased_serde::Deserializer<'de>>,
}

impl<'a, 'de> DynInput<'a, 'de> {
    /// TODO
    pub fn deserialize<T: Deserialize<'de>>(self) -> Result<T, ()> {
        erased_serde::deserialize(self.deserializer.ok_or(())?).map_err(|_| ())
    }

    /// TODO
    pub fn value<T: 'static>(self) -> Option<T> {
        Some(
            self.value?
                .downcast_mut::<Option<T>>()?
                .take()
                // This takes method takes `self` and it's not `Clone` so it's not possible to previously take the value.
                .expect("unreachable"),
        )
    }
}

impl<'a, 'de> fmt::Debug for DynInput<'a, 'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
