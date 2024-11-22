use std::{error, fmt};

use serde::{Serialize, Serializer};

use crate::DeserializeError;

/// TODO
pub enum ProcedureError<S: Serializer> {
    /// Attempted to deserialize a value but failed.
    Deserialize(DeserializeError),
    // /// Attempting to downcast a value failed.
    // Downcast {
    //     // If `None`, the procedure was got a deserializer but expected a value.
    //     // else the name of the type that was provided by the caller.
    //     from: Option<&'static str>,
    //     // The type the procedure expected.
    //     to: &'static str,
    // }, // TODO: Is this going to be possible. Maybe `DowncastError` type?
    /// An error occurred while serializing the value returned by the procedure.
    Serializer(S::Error),
    /// An error occurred while running the procedure.
    Resolver(ResolverError),
}

impl<S: Serializer> From<ResolverError> for ProcedureError<S> {
    fn from(err: ResolverError) -> Self {
        match err.0 {
            Repr::Custom { .. } => ProcedureError::Resolver(err),
            Repr::Deserialize(err) => ProcedureError::Deserialize(err),
            // Repr::Downcast { from, to } => todo!(),
        }
    }
}

impl<S: Serializer> fmt::Debug for ProcedureError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Proper format
        match self {
            Self::Deserialize(err) => write!(f, "Deserialize({:?})", err),
            // Self::Downcast { from, to } => write!(f, "Downcast({:?} -> {:?})", from, to),
            Self::Serializer(err) => write!(f, "Serializer({:?})", err),
            Self::Resolver(err) => write!(f, "Resolver({:?})", err),
        }
    }
}

impl<S: Serializer> fmt::Display for ProcedureError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl<S: Serializer> error::Error for ProcedureError<S> {}

#[derive(Debug)] // TODO: Remove
enum Repr {
    // An actual resolver error.
    Custom {
        status: u16,
        // source: erased_serde::Serialize

        // type_name: &'static str,
        // type_id: TypeId,
        // inner: Box<dyn ErasedError>,
        // #[cfg(debug_assertions)]
        // source_type_name
    },
    // We hide these in here for DX (being able to do `?`) and then convert them to proper `ProcedureError` variants.
    Deserialize(DeserializeError),
    // Downcast {
    //     from: Option<&'static str>,
    //     to: &'static str,
    // },
}

impl From<DeserializeError> for ResolverError {
    fn from(err: DeserializeError) -> Self {
        Self(Repr::Deserialize(err))
    }
}

#[derive(Debug)] // TODO: Custom Debug & std::error::Error
pub struct ResolverError(Repr);

impl ResolverError {
    pub fn new<T: Serialize + 'static, E: error::Error + 'static>(
        status: u16,
        value: T,
        source: Option<E>,
    ) -> Self {
        Self(Repr::Custom {
            status,
            // TODO: Avoid allocing `E` & `T` separately.
            // type_name: type_name::<T>(),
            // type_id: TypeId::of::<T>(),
            // inner: Box::new(value),
        })
    }

    pub(crate) fn _erased_serde(source: erased_serde::Error) -> Self {
        Self(Repr::Deserialize(DeserializeError(source)))
    }

    // pub(crate) fn _downcast(from: Option<&'static str>, to: &'static str) -> Self {
    //     Self(Repr::Downcast { from, to })
    // }
}

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for ResolverError {}

trait ErasedError {
    // fn to_box_any(self: Box<Self>) -> Box<dyn Any>;

    // fn to_value(&self) -> Option<Result<serde_value::Value, serde_value::SerializerError>>;
}

struct ResolverErrorInternal<T: Serialize, E: error::Error + 'static> {
    status: u16,
    value: T,
    source: Option<E>,
}

// impl<T: error::Error + erased_serde::Serialize + 'static> ErasedError for T {
//     // fn to_box_any(self: Box<Self>) -> Box<dyn Any> {
//     //     self
//     // }

//     // fn to_value(&self) -> Option<Result<serde_value::Value, serde_value::SerializerError>> {
//     //     Some(serde_value::to_value(self))
//     // }
// }

// pub struct Serialize(Box<dyn Error>)
