use std::{error, fmt, sync::Arc};

use serde::Serialize;
use specta::Type;

use crate::internal::jsonrpc::JsonRPCError;

#[derive(thiserror::Error, Debug)]
pub enum ExecError {
    #[error("the requested operation '{0}' is not supported by this server")]
    OperationNotFound(String),
    #[error("error deserializing procedure arguments: {0}")]
    DeserializingArgErr(serde_json::Error),
    #[error("error serializing procedure result: {0}")]
    SerializingResultErr(serde_json::Error),
    #[error("error in axum extractor")]
    AxumExtractorError,
    #[error("invalid JSON-RPC version")]
    InvalidJsonRpcVersion,
    #[error("method '{0}' is not supported by this endpoint.")] // TODO: Better error message
    UnsupportedMethod(String),
    #[error("resolver threw error")]
    ErrResolverError(#[from] Error),
    #[error("error creating subscription with null id")]
    ErrSubscriptionWithNullId,
    #[error("error creating subscription with duplicate id")]
    ErrSubscriptionDuplicateId,
}

impl From<ExecError> for Error {
    fn from(v: ExecError) -> Error {
        match v {
            ExecError::OperationNotFound(_) => Error {
                code: ErrorCode::NotFound,
                message: "the requested operation is not supported by this server".to_string(),
                cause: None,
            },
            ExecError::DeserializingArgErr(err) => Error {
                code: ErrorCode::BadRequest,
                message: "error deserializing procedure arguments".to_string(),
                cause: Some(Arc::new(err)),
            },
            ExecError::SerializingResultErr(err) => Error {
                code: ErrorCode::InternalServerError,
                message: "error serializing procedure result".to_string(),
                cause: Some(Arc::new(err)),
            },
            ExecError::AxumExtractorError => Error {
                code: ErrorCode::BadRequest,
                message: "Error running Axum extractors on the HTTP request".into(),
                cause: None,
            },
            ExecError::InvalidJsonRpcVersion => Error {
                code: ErrorCode::BadRequest,
                message: "invalid JSON-RPC version".into(),
                cause: None,
            },
            ExecError::ErrResolverError(err) => err,
            ExecError::UnsupportedMethod(_) => Error {
                code: ErrorCode::BadRequest,
                message: "unsupported metho".into(),
                cause: None,
            },
            ExecError::ErrSubscriptionWithNullId => Error {
                code: ErrorCode::BadRequest,
                message: "error creating subscription with null request id".into(),
                cause: None,
            },
            ExecError::ErrSubscriptionDuplicateId => Error {
                code: ErrorCode::BadRequest,
                message: "error creating subscription with duplicate id".into(),
                cause: None,
            },
        }
    }
}

impl From<ExecError> for JsonRPCError {
    fn from(err: ExecError) -> Self {
        let x: Error = err.into();
        x.into()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ExportError {
    #[error("IO error exporting bindings: {0}")]
    IOErr(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Type)]
#[allow(dead_code)]
pub struct Error {
    pub(crate) code: ErrorCode,
    pub(crate) message: String,
    #[serde(skip)]
    pub(crate) cause: Option<Arc<dyn std::error::Error + Send + Sync>>, // We are using `Arc` instead of `Box` so we can clone the error cause `Clone` isn't dyn safe.
}

impl From<Error> for JsonRPCError {
    fn from(err: Error) -> Self {
        JsonRPCError {
            code: err.code.to_status_code() as i32,
            message: err.message,
            data: None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rspc::Error {{ code: {:?}, message: {} }}",
            self.code, self.message
        )
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl Error {
    pub const fn new(code: ErrorCode, message: String) -> Self {
        Error {
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

/// TODO
#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub enum ErrorCode {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Timeout,
    Conflict,
    PreconditionFailed,
    PayloadTooLarge,
    MethodNotSupported,
    ClientClosedRequest,
    InternalServerError,
}

impl ErrorCode {
    pub fn to_status_code(&self) -> u16 {
        match self {
            ErrorCode::BadRequest => 400,
            ErrorCode::Unauthorized => 401,
            ErrorCode::Forbidden => 403,
            ErrorCode::NotFound => 404,
            ErrorCode::Timeout => 408,
            ErrorCode::Conflict => 409,
            ErrorCode::PreconditionFailed => 412,
            ErrorCode::PayloadTooLarge => 413,
            ErrorCode::MethodNotSupported => 405,
            ErrorCode::ClientClosedRequest => 499,
            ErrorCode::InternalServerError => 500,
        }
    }

    pub const fn from_status_code(status_code: u16) -> Option<Self> {
        match status_code {
            400 => Some(ErrorCode::BadRequest),
            401 => Some(ErrorCode::Unauthorized),
            403 => Some(ErrorCode::Forbidden),
            404 => Some(ErrorCode::NotFound),
            408 => Some(ErrorCode::Timeout),
            409 => Some(ErrorCode::Conflict),
            412 => Some(ErrorCode::PreconditionFailed),
            413 => Some(ErrorCode::PayloadTooLarge),
            405 => Some(ErrorCode::MethodNotSupported),
            499 => Some(ErrorCode::ClientClosedRequest),
            500 => Some(ErrorCode::InternalServerError),
            _ => None,
        }
    }
}
