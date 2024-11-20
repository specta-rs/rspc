use std::{
    any::{type_name, Any, TypeId},
    error, fmt,
};

use serde::{Serialize, Serializer};

use crate::rewrite::Error;

use super::ProcedureOutputSerializeError;

pub enum InternalError {
    /// Attempted to deserialize input but found downcastable input.
    ErrInputNotDeserializable(&'static str),
    /// Attempted to downcast input but found deserializable input.
    ErrInputNotDowncastable,
    /// Error when deserializing input.
    // Boxed to seal `erased_serde` from public API.
    ErrDeserializingInput(Box<dyn std::error::Error>), // TODO: Maybe seal so this type *has* to come from `erased_serde`???
}

impl fmt::Debug for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternalError::ErrInputNotDeserializable(type_name) => {
                write!(f, "input is not deserializable, found type: {type_name}")
            }
            InternalError::ErrInputNotDowncastable => {
                write!(f, "input is not downcastable")
            }
            InternalError::ErrDeserializingInput(err) => {
                write!(f, "failed to deserialize input: {err}")
            }
        }
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for InternalError {}

trait ErasedError: error::Error + erased_serde::Serialize + Any + Send + 'static {
    fn to_box_any(self: Box<Self>) -> Box<dyn Any>;

    fn to_value(&self) -> Option<Result<serde_value::Value, serde_value::SerializerError>>;
}
impl<T: error::Error + Serialize + Any + Send + 'static> ErasedError for T {
    fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }

    fn to_value(&self) -> Option<Result<serde_value::Value, serde_value::SerializerError>> {
        Some(serde_value::to_value(self))
    }
}

pub struct ResolverError {
    status: u16,
    type_name: &'static str,
    type_id: TypeId,
    inner: Box<dyn ErasedError>,
}

impl ResolverError {
    pub fn new<T: Error>(value: T) -> Self {
        Self {
            status: value.status(),
            type_name: type_name::<T>(),
            type_id: TypeId::of::<T>(),
            inner: Box::new(value),
        }
    }

    pub fn status(&self) -> u16 {
        if self.status > 400 || self.status < 600 {
            return 500;
        }

        self.status
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

    // TODO: Using `ProcedureOutputSerializeError`????
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

impl fmt::Debug for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl error::Error for ResolverError {}
