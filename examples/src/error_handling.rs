use std::{error, fmt};

use rspc::{Error, ErrorCode, Router};
use serde::Serialize;
use specta::Type;

use crate::R;

#[derive(thiserror::Error, Serialize, Type, Debug)]
pub enum MyCustomError {
    #[error("I am broke")]
    IAmBroke,
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
            R.error::<MyCustomError>()
                .query(|_, _args: ()| Err::<String, _>(MyCustomError::IAmBroke)),
        )
        .procedure(
            "asyncCustomError",
            R.error::<MyCustomError>()
                .mutation(|_, _args: ()| async move { Err::<String, _>(MyCustomError::IAmBroke) }),
        )
}
