use std::{any::Any, borrow::Cow, error, fmt};

use serde::{
    ser::{Error, SerializeStruct},
    Serialize, Serializer,
};

use crate::LegacyErrorInterop;

// TODO: Discuss the stability guanrantees of the error handling system. Variant is fixed, message is not.

/// TODO
pub enum ProcedureError {
    /// Failed to find a procedure with the given name.
    NotFound,
    /// Attempted to deserialize a value but failed.
    Deserialize(DeserializeError),
    /// Attempting to downcast a value failed.
    Downcast(DowncastError),
    /// An error occurred while running the procedure.
    Resolver(ResolverError),
    /// The procedure unexpectedly unwinded.
    /// This happens when you panic inside a procedure.
    Unwind(Box<dyn Any + Send>),
    // /// An error occurred while serializing the response.
    // /// The error message can be provided should be omitted unless the client is trusted (Eg. Tauri).
    // Serializer(Option<String>), // TODO: Sort this out
}

impl ProcedureError {
    pub fn status(&self) -> u16 {
        match self {
            Self::NotFound => 404,
            Self::Deserialize(_) => 400,
            Self::Downcast(_) => 400,
            Self::Resolver(err) => err.status(),
            Self::Unwind(_) => 500,
            // Self::Serializer(_) => 500,
        }
    }

    pub fn variant(&self) -> &'static str {
        match self {
            ProcedureError::NotFound => "NotFound",
            ProcedureError::Deserialize(_) => "Deserialize",
            ProcedureError::Downcast(_) => "Downcast",
            ProcedureError::Resolver(_) => "Resolver",
            ProcedureError::Unwind(_) => "ResolverPanic",
            // ProcedureError::Serializer(_) => "Serializer",
        }
    }

    // TODO: This should be treated as sanitized and okay for the frontend right?
    pub fn message(&self) -> Cow<'static, str> {
        match self {
            ProcedureError::NotFound => "procedure not found".into(),
            ProcedureError::Deserialize(err) => err.0.to_string().into(),
            ProcedureError::Downcast(err) => err.to_string().into(),
            ProcedureError::Resolver(err) => err
                .error()
                .map(|err| err.to_string().into())
                .unwrap_or("resolver error".into()),
            ProcedureError::Unwind(_) => "resolver panic".into(),
            // ProcedureError::Serializer(err) => err
            //     .clone()
            //     .map(Into::into)
            //     .unwrap_or("serializer error".into()),
        }
    }
}

impl From<ResolverError> for ProcedureError {
    fn from(err: ResolverError) -> Self {
        ProcedureError::Resolver(err)
    }
}

impl From<DeserializeError> for ProcedureError {
    fn from(err: DeserializeError) -> Self {
        ProcedureError::Deserialize(err)
    }
}

impl From<DowncastError> for ProcedureError {
    fn from(err: DowncastError) -> Self {
        ProcedureError::Downcast(err)
    }
}

impl fmt::Debug for ProcedureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Proper format
        match self {
            Self::NotFound => write!(f, "NotFound"),
            Self::Deserialize(err) => write!(f, "Deserialize({err:?})"),
            Self::Downcast(err) => write!(f, "Downcast({err:?})"),
            Self::Resolver(err) => write!(f, "Resolver({err:?})"),
            Self::Unwind(err) => write!(f, "ResolverPanic({err:?})"),
            // Self::Serializer(err) => write!(f, "Serializer({err:?})"),
        }
    }
}

impl fmt::Display for ProcedureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl error::Error for ProcedureError {}

impl Serialize for ProcedureError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let ProcedureError::Resolver(err) = self {
            if let Some(err) = err.error() {
                if let Some(v) = err.downcast_ref::<LegacyErrorInterop>() {
                    return v.0.serialize(serializer);
                }
            }

            return err.value().serialize(serializer);
        }

        let mut state = serializer.serialize_struct("ProcedureError", 3)?;
        state.serialize_field("_rspc", &true)?;
        state.serialize_field("variant", &self.variant())?;
        state.serialize_field("message", &self.message())?;
        state.end()
    }
}

/// TODO
pub struct ResolverError {
    status: u16,
    value: Box<dyn ErrorInternalExt>,
}

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

        Self {
            status,
            value: Box::new(ErrorInternal { value, err: source }),
        }
    }

    /// TODO
    pub fn status(&self) -> u16 {
        self.status
    }

    /// TODO
    pub fn value(&self) -> impl Serialize + '_ {
        self.value.value()
    }

    /// TODO
    pub fn error(&self) -> Option<&(dyn error::Error + Send + 'static)> {
        self.value.error()
    }
}

impl fmt::Debug for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "status: {:?}, error: {:?}",
            self.status,
            self.value.debug()
        )
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

impl DeserializeError {
    pub fn custom<T: fmt::Display>(err: T) -> Self {
        Self(erased_serde::Error::custom(err))
    }
}

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
