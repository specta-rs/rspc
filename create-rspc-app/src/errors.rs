use requestty::ErrorKind;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Prompt Error: {0}")]
    RequestTty(#[from] ErrorKind),

    #[error("Standard IO Error: {0}")]
    StandardIO(#[from] std::io::Error),

    #[error("Rustc Error: {0}")]
    Rustc(#[from] rustc_version::Error),

    #[error("Error: {0}")]
    Standard(#[from] Box<dyn std::error::Error>),

    #[error("Other Error: {0}")]
    Other(String),
}
