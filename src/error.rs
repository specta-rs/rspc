use std::{error, fmt};

#[derive(thiserror::Error, Debug)]
pub enum ExecError {
    #[error("the requested operation '{0}' is not supported by this server")]
    OperationNotFound(String),
    #[error("error deserializing procedure arguments: {0}")]
    DeserializingArgErr(serde_json::Error),
    #[error("error serializing procedure result: {0}")]
    SerializingResultErr(serde_json::Error),
    #[error("resolver threw error")]
    ErrResolverError(#[from] Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ExportError {
    #[error("IO error exporting bindings: {0}")]
    IOErr(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub struct Error {
    code: ErrorCode,
    message: String,
    // cause: Option<Arc<dyn std::error::Error>>, // We are using `Arc` instead of `Box` so we can clone the error cause `Clone` isn't dyn safe.
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
            // cause: None,
        }
    }

    // pub fn with_cause<TErr: std::error::Error + 'static>(
    //     code: ErrorCode,
    //     message: String,
    //     cause: TErr,
    // ) -> Self {
    //     Self {
    //         code,
    //         message,
    //         cause: Some(Arc::new(cause)),
    //     }
    // }
}

/// TODO
#[derive(Debug, Clone)]
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