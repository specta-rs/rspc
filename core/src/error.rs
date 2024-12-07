use std::{error, fmt};

use serde::{Serialize, Serializer};

/// TODO
pub enum ProcedureError<S: Serializer> {
    /// Attempted to deserialize a value but failed.
    Deserialize(DeserializeError),
    /// Attempting to downcast a value failed.
    Downcast(DowncastError),
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
            Repr::Downcast(downcast) => ProcedureError::Downcast(downcast),
        }
    }
}

impl<S: Serializer> fmt::Debug for ProcedureError<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Proper format
        match self {
            Self::Deserialize(err) => write!(f, "Deserialize({:?})", err),
            Self::Downcast(err) => write!(f, "Downcast({:?})", err),
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

enum Repr {
    // An actual resolver error.
    Custom {
        status: u16,
        value: Box<dyn ErrorInternalExt>,
    },
    // We hide these in here for DX (being able to do `?`) and then convert them to proper `ProcedureError` variants.
    Deserialize(DeserializeError),
    Downcast(DowncastError),
}

impl From<DowncastError> for ResolverError {
    fn from(err: DowncastError) -> Self {
        Self(Repr::Downcast(err))
    }
}

/// TODO
pub struct ResolverError(Repr);

impl ResolverError {
    pub fn new<T: Serialize + Send + 'static, E: error::Error + Send + 'static>(
        status: u16,
        value: T,
        source: Option<E>,
    ) -> Self {
        Self(Repr::Custom {
            status,
            value: Box::new(ErrorInternal { value, err: source }),
        })
    }

    /// TODO
    pub fn status(&self) -> u16 {
        match &self.0 {
            Repr::Custom { status, value: _ } => *status,
            // We flatten these to `ResolverError` so this won't be hit.
            Repr::Deserialize(_) => unreachable!(),
            Repr::Downcast(_) => unreachable!(),
        }
    }

    /// TODO
    pub fn value(&self) -> &dyn erased_serde::Serialize {
        match &self.0 {
            Repr::Custom {
                status: _,
                value: error,
            } => error.value(),
            // We flatten these to `ResolverError` so this won't be hit.
            Repr::Deserialize(_) => unreachable!(),
            Repr::Downcast(_) => unreachable!(),
        }
    }

    /// TODO
    pub fn error(&self) -> Option<&(dyn error::Error + Send + 'static)> {
        match &self.0 {
            Repr::Custom {
                status: _,
                value: error,
            } => error.error(),
            // We flatten these to `ResolverError` so this won't be hit.
            Repr::Deserialize(_) => unreachable!(),
            Repr::Downcast(_) => unreachable!(),
        }
    }
}

impl fmt::Debug for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Repr::Custom {
                status,
                value: error,
            } => {
                write!(f, "status: {status:?}, error: {:?}", error.debug())
            }
            // In practice these won't be hit.
            Repr::Deserialize(err) => write!(f, "Deserialize({err:?})"),
            Repr::Downcast(err) => write!(f, "Downcast({err:?})"),
        }
    }
}

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for ResolverError {}

/// TODO
pub struct DeserializeError(pub(crate) erased_serde::Error);

impl fmt::Debug for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Deserialize({:?})", self.0)
    }
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for DeserializeError {}

impl From<DeserializeError> for ResolverError {
    fn from(err: DeserializeError) -> Self {
        Self(Repr::Deserialize(err))
    }
}

/// TODO
pub struct DowncastError {
    // If `None`, the procedure was got a deserializer but expected a value.
    // else the name of the type that was provided by the caller.
    pub(crate) from: Option<&'static str>,
    // The type the procedure expected.
    pub(crate) to: &'static str,
}

impl fmt::Debug for DowncastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Downcast(from: {:?}, to: {:?})", self.from, self.to)
    }
}

impl fmt::Display for DowncastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for DowncastError {}

struct ErrorInternal<T, E> {
    value: T,
    err: Option<E>,
}

trait ErrorInternalExt: Send {
    fn value(&self) -> &dyn erased_serde::Serialize;

    fn error(&self) -> Option<&(dyn error::Error + Send + 'static)>;

    fn debug(&self) -> Option<&dyn fmt::Debug>;
}

impl<T: Serialize + Send + 'static, E: error::Error + Send + 'static> ErrorInternalExt
    for ErrorInternal<T, E>
{
    fn value(&self) -> &dyn erased_serde::Serialize {
        &self.value
    }

    fn error(&self) -> Option<&(dyn error::Error + Send + 'static)> {
        self.err
            .as_ref()
            .map(|err| err as &(dyn error::Error + Send + 'static))
    }

    fn debug(&self) -> Option<&dyn fmt::Debug> {
        self.err.as_ref().map(|err| err as &dyn fmt::Debug)
    }
}
