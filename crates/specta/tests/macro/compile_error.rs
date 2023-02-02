//! This file is run with the `trybuild` crate to assert compilation errors in the Specta macros.

use specta::Type;

// Invalid inflection
#[derive(Type)]
#[serde(rename_all = "camelCase123")]
pub enum Demo2 {}

// Specta doesn't support Trait objects
#[derive(Type)]
pub struct Error {
    pub(crate) cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

// Enums can only flatten if
// at least one of their variants can flatten

#[derive(Type)]
enum UnitExternal {
    Unit,
}

#[derive(Type)]
enum UnnamedMultiExternal {
    UnnamedMulti(String, String),
}

#[derive(Type)]
struct FlattenExternal {
    #[serde(flatten)]
    unit: UnitExternal,
    #[serde(flatten)]
    unnamed_multi: UnnamedMultiExternal,
}

#[derive(Type)]
#[serde(untagged)]
enum UnnamedUntagged {
    Unnamed(String),
}

#[derive(Type)]
#[serde(untagged)]
enum UnnamedMultiUntagged {
    Unnamed(String, String),
}

#[derive(Type)]
struct FlattenUntagged {
    #[serde(flatten)]
    unnamed: UnnamedUntagged,
    #[serde(flatten)]
    unnamed_multi: UnnamedMultiUntagged,
}

// Adjacent can always flatten

#[derive(Type)]
#[serde(tag = "tag")]
enum UnnamedInternal {
    Unnamed(String),
}

// Internal can't be used with unnamed multis

#[derive(Type)]
struct FlattenInternal {
    #[serde(flatten)]
    unnamed: UnnamedInternal,
}

// TODO: https://docs.rs/trybuild/latest/trybuild/#what-to-test
