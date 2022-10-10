use requestty::ErrorKind;
use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("Request TTY Error: {0}")]
    RequestTtyError(#[from] ErrorKind),
    // #[error("Standard IO ")]
}
