use std::{error, fmt};

pub enum InternalError {
    /// Attempted to deserialize input but found downcastable input.
    ErrInputNotDeserializable(&'static str),
    /// Attempted to downcast input but found deserializable input.
    ErrInputNotDowncastable,
    /// Error when deserializing input.
    // Boxed to seal `erased_serde` from public API.
    ErrDeserializingInput(Box<dyn std::error::Error>),
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
