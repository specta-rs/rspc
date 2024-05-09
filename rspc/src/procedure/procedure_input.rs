use std::{
    any::{type_name, Any, TypeId},
    marker::PhantomData,
};

use serde::de::DeserializeOwned;

use super::Argument;

pub(super) trait InputValueInner<'de> {
    fn into_deserializer(&mut self) -> Option<&mut dyn erased_serde::Deserializer<'de>>;

    fn get_type_name(&self) -> Option<&'static str> {
        None
    }

    fn get_type_id(&self) -> Option<TypeId> {
        None
    }

    fn into_dyn_any(&mut self) -> Option<&mut dyn Any> {
        None
    }
}

pub(super) struct AnyInput<T>(pub Option<T>);
impl<'de, T: Any + 'static> InputValueInner<'de> for AnyInput<T> {
    fn into_deserializer(&mut self) -> Option<&mut dyn erased_serde::Deserializer<'de>> {
        None
    }

    fn get_type_name(&self) -> Option<&'static str> {
        Some(type_name::<T>())
    }

    fn get_type_id(&self) -> Option<TypeId> {
        Some(TypeId::of::<T>())
    }

    fn into_dyn_any(&mut self) -> Option<&mut dyn Any> {
        Some(&mut self.0)
    }
}

impl<'de, D: erased_serde::Deserializer<'de>> InputValueInner<'de> for D {
    fn into_deserializer(&mut self) -> Option<&mut dyn erased_serde::Deserializer<'de>> {
        Some(self)
    }
}

pub struct ProcedureInput<'a, 'b, T>(&'a mut dyn InputValueInner<'b>, PhantomData<T>);

impl<'a, 'b, T> ProcedureInput<'a, 'b, T> {
    pub(crate) fn new(value: &'a mut dyn InputValueInner<'b>) -> Self {
        Self(value, PhantomData)
    }

    pub fn type_name(&self) -> Option<&'static str> {
        self.0.get_type_name()
    }

    pub fn type_id(&self) -> Option<TypeId> {
        self.0.get_type_id()
    }

    // TODO: Should we have a generic downcast???? -> This is typesafe but it means the `TypeId` stuff can't be used for matching???
    pub fn downcast(self) -> Option<T>
    where
        T: Argument<'b> + 'static,
    {
        Some(
            self.0
                .into_dyn_any()?
                .downcast_mut::<Option<T>>()
                .expect("todo: this is typesafe")
                .take()
                .expect("value already taken"),
        )
    }

    pub fn deserialize<U: DeserializeOwned>(self) -> Result<U, ()> {
        erased_serde::deserialize(
            self.0
                .into_deserializer()
                // TODO: Not serde type
                .unwrap(),
        )
        .map_err(|_| ())
    }
}
