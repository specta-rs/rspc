use std::any::TypeId;

use crate::DataType;

/// this is used internally to represent the types.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ObjectField {
    pub name: &'static str,
    pub ty: DataType,
    pub optional: bool,
    pub flatten: bool,
}

/// this is used internally to represent the types.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct ObjectType {
    pub name: &'static str,
    pub generics: Vec<&'static str>,
    pub fields: Vec<ObjectField>,
    pub tag: Option<&'static str>,
    pub type_id: Option<TypeId>,
}

impl PartialEq for ObjectType {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}
