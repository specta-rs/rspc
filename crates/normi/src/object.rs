use serde::Serialize;
use serde_json::Value;
use specta::Type;
use std::any::type_name;

/// TODO
pub trait Object: 'static {
    type NormalizedResult: Serialize + Type + Send;

    /// is used to determine the type of the current object. It will define to Rust's debug type name but you SHOULD override it.
    fn type_name() -> &'static str {
        type_name::<Self>()
    }

    /// is used to determine the unique identifier for this object. The id must be unique between all objects of the same type.
    fn id(&self) -> Value;

    /// TODO
    fn normalize(self) -> Self::NormalizedResult;
}
