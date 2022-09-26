use std::{error, fmt};

use rspc::{Error, ErrorCode, Router, RouterBuilder};

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
pub fn mount() -> RouterBuilder {
    Router::new()
    .query("ok", |t| {
        t(|_, _args: ()| Ok("Hello World".into()) as Result<String, Error>)
    })
    .query("err", |t| {
        t(|_, _args: ()| {
            Err(Error::new(
                ErrorCode::BadRequest,
                "This is a custom error!".into(),
            )) as Result<String, _>
        })
    })
    .query("errWithCause", |t| {
        t(|_, _args: ()| {
            Err(Error::with_cause(
                ErrorCode::BadRequest,
                "This is a custom error!".into(),
                CustomRustError::GenericError,
            )) as Result<String, Error>
        })
    })
    .query("customErr", |t| {
        t(|_, _args: ()| Ok(Err(MyCustomError::IAmBroke)?))
    })
    .query("customErrUsingInto", |t| {
        t(|_, _args: ()| Err(MyCustomError::IAmBroke.into()) as Result<String, Error>)
    })
    .query("asyncCustomError", |t| {
        t(
            |_, _args: ()| async move {
                Err(MyCustomError::IAmBroke.into()) as Result<String, _>
            },
        )
    })
}
