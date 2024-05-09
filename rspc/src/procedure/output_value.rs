use std::any::{type_name, Any, TypeId};

use serde::{de::DeserializeOwned, Serialize, Serializer};
use serde_value::DeserializerError;

trait Inner: Any + 'static {
    fn to_box_any(self: Box<Self>) -> Box<dyn Any>;

    fn to_value(&self) -> Option<Result<serde_value::Value, serde_value::SerializerError>> {
        None
    }
}

struct AnyT<T>(T);
impl<T: Any + 'static> Inner for AnyT<T> {
    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self.0)
    }
}

impl<T: Serialize + Any + 'static> Inner for T {
    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(self)
    }

    fn to_value(&self) -> Option<Result<serde_value::Value, serde_value::SerializerError>> {
        Some(serde_value::to_value(self))
    }
}

pub struct ProcedureResult {
    type_map: &'static str,
    type_id: TypeId,
    inner: Box<dyn Inner>,
}

impl ProcedureResult {
    pub fn new<T: Any + 'static>(value: T) -> Self {
        Self {
            type_map: type_name::<T>(),
            type_id: TypeId::of::<T>(),
            inner: Box::new(AnyT(value)),
        }
    }

    pub fn with_serde<T: Serialize + Any + 'static>(value: T) -> Self {
        Self {
            type_map: type_name::<T>(),
            type_id: TypeId::of::<T>(),
            inner: Box::new(value),
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_map
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn downcast<T: Any>(self) -> Option<T> {
        self.inner.to_box_any().downcast().map(|v| *v).ok()
    }

    pub fn serialize<S: Serializer>(self, ser: S) -> Result<S::Ok, ()> {
        let value = self
            .inner
            .to_value()
            // TODO: This value doesn't support Serde error
            .ok_or(())?
            // TODO: This value had a Serde error
            .map_err(|_| ())?;

        value.serialize(ser).map_err(|_| ())
    }

    pub fn deserialize<T: DeserializeOwned>(self) -> Result<T, ()> {
        let value = self
            .inner
            .to_value()
            // TODO: This value doesn't support Serde error
            .ok_or(())?
            // TODO: This value had a Serde error
            .map_err(|_| ())?;

        T::deserialize(serde_value::ValueDeserializer::<DeserializerError>::new(
            value,
        ))
        .map_err(|_| ())
    }
}
