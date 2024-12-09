use std::{borrow::Cow, error, fmt};

use serde::{ser::SerializeStruct, Serialize, Serializer};

/// TODO
pub enum ProcedureError<S: Serializer> {
    /// Failed to find a procedure with the given name.
    NotFound,
    /// Attempted to deserialize a value but failed.
    Deserialize(DeserializeError),
    /// Attempting to downcast a value failed.
    Downcast(DowncastError),
    /// An error occurred while serializing the value returned by the procedure.
    Serializer(S::Error),
    /// An error occurred while running the procedure.
    Resolver(ResolverError),
}

impl<S: Serializer> ProcedureError<S> {
    pub fn code(&self) -> u16 {
        match self {
            Self::NotFound => 404,
            Self::Deserialize(_) => 400,
            Self::Downcast(_) => 400,
            Self::Serializer(_) => 500,
            Self::Resolver(err) => err.status(),
        }
    }

    pub fn serialize<Se: Serializer>(&self, s: Se) -> Result<Se::Ok, Se::Error> {
        match self {
            Self::NotFound => s.serialize_none(),
            Self::Deserialize(err) => s.serialize_str(&format!("{}", err)),
            Self::Downcast(err) => s.serialize_str(&format!("{}", err)),
            Self::Serializer(err) => s.serialize_str(&format!("{}", err)),
            Self::Resolver(err) => s.serialize_str(&format!("{}", err)),
        }
    }

    pub fn variant(&self) -> &'static str {
        match self {
            ProcedureError::NotFound => "NotFound",
            ProcedureError::Deserialize(_) => "Deserialize",
            ProcedureError::Downcast(_) => "Downcast",
            ProcedureError::Serializer(_) => "Serializer",
            ProcedureError::Resolver(_) => "Resolver",
        }
    }

    pub fn message(&self) -> Cow<'static, str> {
        match self {
            ProcedureError::NotFound => "procedure not found".into(),
            ProcedureError::Deserialize(err) => err.0.to_string().into(),
            ProcedureError::Downcast(err) => err.to_string().into(),
            ProcedureError::Serializer(err) => err.to_string().into(),
            ProcedureError::Resolver(err) => err
                .error()
                .map(|err| err.to_string().into())
                .unwrap_or("resolver error".into()),
        }
    }
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
            Self::NotFound => write!(f, "NotFound"),
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

impl<Se: Serializer> Serialize for ProcedureError<Se> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let ProcedureError::Resolver(err) = self {
            return err.value().serialize(serializer);
        }

        let mut state = serializer.serialize_struct("ProcedureError", 1)?;
        state.serialize_field("_rspc", &true)?;
        state.serialize_field("variant", &self.variant())?;
        state.serialize_field("message", &self.message())?;
        state.end()
    }
}

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
    // Warning: Returning > 400 will fallback to `500`. As redirects would be invalid and `200` would break matching.
    pub fn new<T: Serialize + Send + 'static, E: error::Error + Send + 'static>(
        mut status: u16,
        value: T,
        source: Option<E>,
    ) -> Self {
        if status < 400 {
            status = 500;
        }

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
    pub fn value(&self) -> impl Serialize + '_ {
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
