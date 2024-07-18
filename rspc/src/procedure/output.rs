use std::{
    any::{type_name, Any, TypeId},
    error, fmt,
};

use serde::{Serialize, Serializer};

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

pub struct ProcedureOutput {
    type_name: &'static str,
    type_id: TypeId,
    inner: Box<dyn Inner + Send>,
}

impl fmt::Debug for ProcedureOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProcedureOutput")
            .field("type_name", &self.type_name)
            .field("type_id", &self.type_id)
            .finish()
    }
}

impl ProcedureOutput {
    pub fn new<T: Any + Send + 'static>(value: T) -> Self {
        Self {
            type_name: type_name::<T>(),
            type_id: TypeId::of::<T>(),
            inner: Box::new(AnyT(value)),
        }
    }

    pub fn with_serde<T: Serialize + Any + Send + 'static>(value: T) -> Self {
        Self {
            type_name: type_name::<T>(),
            type_id: TypeId::of::<T>(),
            inner: Box::new(value),
        }
    }

    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn downcast<T: Any>(self) -> Option<T> {
        self.inner.to_box_any().downcast().map(|v| *v).ok()
    }

    pub fn serialize<S: Serializer>(
        self,
        ser: S,
    ) -> Result<S::Ok, ProcedureOutputSerializeError<S>> {
        let value = self
            .inner
            .to_value()
            .ok_or(ProcedureOutputSerializeError::ErrResultNotDeserializable(
                self.type_name,
            ))?
            .expect("serde_value doesn't panic"); // TODO: This is false

        value
            .serialize(ser)
            .map_err(ProcedureOutputSerializeError::ErrSerializer)
    }
}

pub enum ProcedureOutputSerializeError<S: Serializer> {
    /// Attempted to deserialize input but found downcastable input.
    ErrResultNotDeserializable(&'static str),
    /// Error occurred in the serializer you provided.
    ErrSerializer(S::Error),
}

impl<S: Serializer> fmt::Debug for ProcedureOutputSerializeError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ErrResultNotDeserializable(type_name) => {
                write!(f, "Result type {type_name} is not deserializable")
            }
            Self::ErrSerializer(err) => write!(f, "Serializer error: {err:?}"),
        }
    }
}

impl<S: Serializer> fmt::Display for ProcedureOutputSerializeError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<S: Serializer> error::Error for ProcedureOutputSerializeError<S> {}
