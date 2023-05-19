use std::{error, fmt};

use rspc::{Error, ErrorCode, Router};

use crate::R;

pub enum MyCustomError {
    IAmBroke,
}

impl From<MyCustomError> for Error {
    fn from(_: MyCustomError) -> Self {
        Error::new(ErrorCode::InternalServerError, "I am broke".into())
    }
}

#[derive(Debug)]
pub enum CustomRustError {
    GenericError,
}

impl fmt::Display for CustomRustError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "some Rust error!")
    }
}

impl error::Error for CustomRustError {}

// We merge this router into the main router in `main.rs`.
// This router shows how to do error handling
pub fn mount() -> Router<()> {
    R.router()
        .procedure(
            "ok",
            R.query(|_, _args: ()| Ok("Hello World".into()) as Result<String, Error>),
        )
        .procedure(
            "err",
            R.query(|_, _args: ()| {
                Err(Error::new(
                    ErrorCode::BadRequest,
                    "This is a custom error!".into(),
                )) as Result<String, _>
            }),
        )
        .procedure(
            "errWithCause",
            R.query(|_, _args: ()| {
                Err(Error::with_cause(
                    ErrorCode::BadRequest,
                    "This is a custom error!".into(),
                    CustomRustError::GenericError,
                )) as Result<String, Error>
            }),
        )
        .procedure(
            "customErr",
            R.query(|_, _args: ()| Ok(Err(MyCustomError::IAmBroke)?)),
        )
        .procedure(
            "customErrUsingInto",
            R.query(|_, _args: ()| Err(MyCustomError::IAmBroke.into()) as Result<String, Error>),
        )
        .procedure(
            "asyncCustomError",
            R.mutation(|_, _args: ()| async move {
                Err(MyCustomError::IAmBroke.into()) as Result<String, _>
            }),
        )
}
