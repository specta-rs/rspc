use serde::Serialize;
use serde_json::Value;
use specta::Type;

/// TODO
pub trait Object: 'static {
    type NormalizedResult: Serialize + Type + Send;

    /// is used to determine the type of the current object. It will define to Rust's debug type name but you SHOULD override it.
    fn type_name() -> &'static str;

    /// is used to determine the unique identifier for this object. The id must be unique between all objects of the same type.
    fn id(&self) -> Result<Value, serde_json::Error>;

    /// TODO
    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error>;
}

#[derive(Serialize, Type)]
pub struct NormalizedVec<T> {
    __type: &'static str,
    edges: Vec<T>,
}

impl<T: Object + Serialize + Type + Send> Object for Vec<T> {
    type NormalizedResult = NormalizedVec<T>;

    fn type_name() -> &'static str {
        T::type_name()
    }

    fn id(&self) -> Result<Value, serde_json::Error> {
        unreachable!();
    }

    fn normalize(self) -> Result<Self::NormalizedResult, serde_json::Error> {
        Ok(NormalizedVec {
            __type: Self::type_name(),
            edges: self,
        })
    }
}

// TODO: All other common container types
