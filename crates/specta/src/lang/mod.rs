/// Alpha: [OpenAPI](https://www.openapis.org) support.
#[cfg(feature = "openapi")]
#[allow(missing_docs)] // TODO: Remove once this is stable
pub mod openapi;

/// [Typescript](https://www.typescriptlang.org) support.
#[cfg(feature = "typescript")]
pub mod ts;
