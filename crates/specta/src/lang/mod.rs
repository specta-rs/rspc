// /// Alpha: [OpenAPI](https://www.openapis.org) support.
// #[cfg(feature = "openapi")]
// #[allow(missing_docs)] // TODO: Remove once this is stable
// #[allow(warnings)] // TODO: Remove once this is out of dev
// pub mod openapi;

/// [Typescript](https://www.typescriptlang.org) support.
#[cfg(feature = "typescript")]
pub mod ts;

// /// [Rust](https://www.rust-lang.org) support.
// #[cfg(feature = "rust")]
// pub mod rust;

// /// [Swift](https://www.swift.org) support.
// #[cfg(feature = "swift")]
// pub mod swift;

// /// [Kotlin](https://kotlinlang.org) support.
// #[cfg(feature = "kotlin")]
// pub mod kotlin;

// /// [Go Lang](https://go.dev) support.
// #[cfg(feature = "go")]
// pub mod go;

macro_rules! primitive_def {
    ($($t:ident)+) => {
        $(DataType::Primitive(PrimitiveType::$t))|+
    }
}

pub(crate) use primitive_def;
