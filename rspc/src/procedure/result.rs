use std::{
    any::{Any, TypeId},
    future::ready,
    pin::Pin,
};

use serde::{Serialize, Serializer};

use super::erased_fut::{AnyErasedFut, ErasedFut};

pub struct ProcedureResult {
    type_id: std::any::TypeId,
    inner: Pin<Box<dyn AnyErasedFut>>,
}

impl ProcedureResult {
    pub fn new<T: Any + 'static>(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            inner: Box::pin(ErasedFut::Execute(ready(value))), // TODO: `ready` is not right
        }
    }

    pub fn with_serde<T: Serialize + Any + 'static>(value: T) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            inner: Box::pin(ErasedFut::Execute(ready(value))), // TODO: `ready` is not right
        }
    }

    pub fn type_id(&self) -> std::any::TypeId {
        self.type_id
    }

    pub fn downcast<T: Any>(mut self) -> Option<T> {
        // TODO: Ensure we have polled it to completion before here

        Some(
            self.inner
                .as_mut()
                .take_any()
                .downcast_mut::<Option<T>>()?
                .take()
                .expect("value has already been taken"),
        )
    }

    pub fn serialize<S: Serializer>(mut self, ser: S) -> Result<(), ()> {
        // TODO: Ensure we have polled it to completion before here

        self.inner
            .as_mut()
            .take_serde()
            .ok_or(())? // TODO: This value doesn't support Serde error
            .erased_serialize(&mut <dyn erased_serde::Serializer>::erase(ser))
            .map_err(|_| ())
    }
}
