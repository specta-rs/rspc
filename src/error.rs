use thiserror::Error;

/// TODO
#[derive(Error, Debug)]
pub enum ExecError {
    #[error("the requested operation is not supported by this server")]
    OperationNotFound,
    #[error("error serialising the result of the operation")]
    ErrSerialiseResult(serde_json::Error),
    #[error("error deserialising the argument for the operation")]
    ErrDeserialiseArg(serde_json::Error),
    #[error("error `rspc` got into an unreachable state. Please report this issue to developers!")]
    UnreachableInternalState,
}
