use std::any::{Any, TypeId};

use serde::{Serialize, Serializer};

// Rust doesn't allow `+` with `dyn` for non-auto traits.
trait ErasedSerdeSerializePlusAny: erased_serde::Serialize + Any + 'static {
    /// `downcast` is implemented for `Box<dyn Any>` so we need to upcast
    fn to_box(self: Box<Self>) -> Box<dyn Any>;
}
impl<T> ErasedSerdeSerializePlusAny for T
where
    T: erased_serde::Serialize + Any + 'static,
{
    fn to_box(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

enum Inner {
    Any(Box<dyn Any>),
    Serde(Box<dyn ErasedSerdeSerializePlusAny>),
}

pub struct ProcedureResult {
    type_id: std::any::TypeId,
    inner: Inner,
}

impl ProcedureResult {
    pub fn new<T: Any + 'static>(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            inner: Inner::Any(Box::new(value)),
        }
    }

    pub fn with_serde<T: Serialize + Any + 'static>(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            inner: Inner::Serde(Box::new(value)),
        }
    }

    pub fn type_id(&self) -> std::any::TypeId {
        self.type_id
    }

    pub fn downcast<T: Any>(self) -> Option<T> {
        match self.inner {
            Inner::Any(v) => v,
            Inner::Serde(v) => v.to_box(),
        }
        .downcast()
        .map(|v| *v)
        .ok()
    }

    pub fn serialize<S: Serializer>(self, ser: S) -> Result<S::Ok, ()> {
        // match self.inner {
        //     Inner::Any(_) => Err(()), // TODO: This value doesn't support Serde error
        //     Inner::Serde(v) => v
        //         .erased_serialize(&mut <dyn erased_serde::Serializer>::erase(ser))
        //         .map_err(|_| ()),
        // }
        todo!();
    }
}
