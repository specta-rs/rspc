//! Support for [`validator`] with [`rspc`] for easy input validation.
#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(
    html_logo_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true",
    html_favicon_url = "https://github.com/specta-rs/rspc/blob/main/.github/logo.png?raw=true"
)]

use std::fmt;

use rspc::middleware::Middleware;
use serde::{ser::SerializeStruct, Serialize};
use specta::{datatype::DataType, Type};
use validator::{Validate, ValidationErrors};

/// TODO
pub fn validate<TError, TCtx, TInput, TResult>() -> Middleware<TError, TCtx, TInput, TResult>
where
    TError: From<RspcValidatorError> + Send + 'static,
    TCtx: Send + 'static,
    TInput: Validate + Send + 'static,
    TResult: Send + 'static,
{
    Middleware::new(|ctx, input: TInput, next| async move {
        match input.validate() {
            Ok(()) => next.exec(ctx, input).await,
            Err(err) => Err(RspcValidatorError(err).into()),
        }
    })
}

#[derive(Clone)]
pub struct RspcValidatorError(ValidationErrors);

impl fmt::Debug for RspcValidatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl fmt::Display for RspcValidatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl std::error::Error for RspcValidatorError {}

impl Serialize for RspcValidatorError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("RspcValidatorError", 2)?;
        s.serialize_field("~rspc.validator", &true)?;
        s.serialize_field("errors", &self.0.field_errors())?;
        s.end()
    }
}

// TODO: Proper implementation
impl Type for RspcValidatorError {
    fn inline(
        _type_map: &mut specta::TypeCollection,
        _generics: specta::Generics,
    ) -> specta::datatype::DataType {
        DataType::Any
    }
}
