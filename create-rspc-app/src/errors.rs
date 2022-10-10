use requestty::ErrorKind;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Prompt Error: {0}")]
    RequestTtyError(#[from] ErrorKind),

    #[error("Standard IO Error: {0}")]
    StandardIOError(#[from] std::io::Error),

    #[error("Rustc Error: {0}")]
    RustcError(#[from] rustc_version::Error),

    #[error("Standard Error: {0}")]
    StandardError(#[from] Box<dyn std::error::Error>),

    #[error("Other Error: {0}")]
    Other(String),
}
