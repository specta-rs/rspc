use std::{error, fmt, sync::Arc};

use serde::Serialize;
use specta::Type;

use crate::{internal::jsonrpc::JsonRPCError, ErrorCode};

#[derive(thiserror::Error, Debug)]
// #[non_exhaustive] // TODO
pub enum AlphaExecError {
    #[error("the requested operation '{0}' is not supported by this server")]
    OperationNotFound(String),
    #[error("error deserializing procedure arguments: {0}")]
    DeserializingArgErr(serde_json::Error),
    #[error("error serializing procedure result: {0}")]
    SerializingResultErr(serde_json::Error),
    #[cfg(feature = "axum")]
    #[error("error in axum extractor")]
    AxumExtractorError,
    #[error("invalid JSON-RPC version")]
    InvalidJsonRpcVersion,
    #[error("method '{0}' is not supported by this endpoint.")] // TODO: Better error message
    UnsupportedMethod(String),
    #[error("resolver threw error")]
    ErrResolverError(#[from] AlphaError),
    #[error("error creating subscription with null id")]
    ErrSubscriptionWithNullId,
    #[error("error creating subscription with duplicate id")]
    ErrSubscriptionDuplicateId,
}

impl From<AlphaExecError> for AlphaError {
    fn from(v: AlphaExecError) -> AlphaError {
        match v {
            AlphaExecError::OperationNotFound(_) => AlphaError {
                code: ErrorCode::NotFound,
                message: "the requested operation is not supported by this server".to_string(),
                cause: None,
            },
            AlphaExecError::DeserializingArgErr(err) => AlphaError {
                code: ErrorCode::BadRequest,
                message: "error deserializing procedure arguments".to_string(),
                cause: Some(Arc::new(err)),
            },
            AlphaExecError::SerializingResultErr(err) => AlphaError {
                code: ErrorCode::InternalServerError,
                message: "error serializing procedure result".to_string(),
                cause: Some(Arc::new(err)),
            },
            #[cfg(feature = "axum")]
            AlphaExecError::AxumExtractorError => AlphaError {
                code: ErrorCode::BadRequest,
                message: "Error running Axum extractors on the HTTP request".into(),
                cause: None,
            },
            AlphaExecError::InvalidJsonRpcVersion => AlphaError {
                code: ErrorCode::BadRequest,
                message: "invalid JSON-RPC version".into(),
                cause: None,
            },
            AlphaExecError::ErrResolverError(err) => err,
            AlphaExecError::UnsupportedMethod(_) => AlphaError {
                code: ErrorCode::BadRequest,
                message: "unsupported metho".into(),
                cause: None,
            },
            AlphaExecError::ErrSubscriptionWithNullId => AlphaError {
                code: ErrorCode::BadRequest,
                message: "error creating subscription with null request id".into(),
                cause: None,
            },
            AlphaExecError::ErrSubscriptionDuplicateId => AlphaError {
                code: ErrorCode::BadRequest,
                message: "error creating subscription with duplicate id".into(),
                cause: None,
            },
        }
    }
}

impl From<AlphaExecError> for JsonRPCError {
    fn from(err: AlphaExecError) -> Self {
        let x: AlphaError = err.into();
        x.into()
    }
}

#[derive(Debug, Clone, Serialize, Type)]
#[allow(dead_code)]
pub struct AlphaError {
    pub(crate) code: ErrorCode,
    pub(crate) message: String,
    #[serde(skip)]
    pub(crate) cause: Option<Arc<dyn std::error::Error + Send + Sync>>, // We are using `Arc` instead of `Box` so we can clone the error cause `Clone` isn't dyn safe.
}

impl From<AlphaError> for JsonRPCError {
    fn from(err: AlphaError) -> Self {
        JsonRPCError {
            code: err.code.to_status_code() as i32,
            message: err.message,
            data: None,
        }
    }
}

impl fmt::Display for AlphaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rspc::Error {{ code: {:?}, message: {} }}",
            self.code, self.message
        )
    }
}

impl error::Error for AlphaError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl AlphaError {
    pub const fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            cause: None,
        }
    }

    pub fn with_cause<TErr>(code: ErrorCode, message: String, cause: TErr) -> Self
    where
        TErr: std::error::Error + Send + Sync + 'static,
    {
        Self {
            code,
            message,
            cause: Some(Arc::new(cause)),
        }
    }
}

#[cfg(feature = "anyhow")]
impl From<anyhow::Error> for AlphaError {
    fn from(_value: anyhow::Error) -> Self {
        AlphaError {
            code: ErrorCode::InternalServerError,
            message: "internal server error".to_string(),
            cause: None, // TODO: Make this work
        }
    }
}

impl From<AlphaError> for crate::Error {
    fn from(err: AlphaError) -> Self {
        crate::Error::new(err.code, err.message)
    }
}
