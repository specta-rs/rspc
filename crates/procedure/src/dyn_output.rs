use std::{
    any::{type_name, Any},
    fmt,
};

use serde::Serialize;

use crate::ProcedureError;

/// TODO
pub struct DynOutput<'a> {
    inner: Repr<'a>,
    pub(crate) type_name: &'static str,
}

enum Repr<'a> {
    Serialize(&'a (dyn erased_serde::Serialize + Send + Sync)),
    Value(&'a mut (dyn Any + Send)),
}

// TODO: `Debug`, etc traits

impl<'a> DynOutput<'a> {
    pub fn new_value<T: Send + 'static>(value: &'a mut Option<T>) -> Self {
        Self {
            inner: Repr::Value(value),
            type_name: type_name::<T>(),
        }
    }

    pub fn new_serialize<T: Serialize + Send + Sync>(value: &'a mut T) -> Self {
        Self {
            inner: Repr::Serialize(value),
            type_name: type_name::<T>(),
        }
    }

    /// TODO
    pub fn as_serialize(self) -> Option<impl Serialize + Send + Sync + 'a> {
        match self.inner {
            Repr::Serialize(v) => Some(v),
            Repr::Value(_) => None,
        }
    }

    /// TODO
    pub fn as_value<T: Send + 'static>(self) -> Option<T> {
        match self.inner {
            Repr::Serialize(_) => None,
            Repr::Value(v) => v
                .downcast_mut::<Option<Result<_, ProcedureError>>>()?
                .take()
                .expect("unreachable")
                .expect("unreachable"),
        }
    }
}

impl<'a> fmt::Debug for DynOutput<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
