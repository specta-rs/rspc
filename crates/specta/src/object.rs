use std::any::TypeId;

use crate::DataType;

#[derive(Debug, Clone)]
pub struct ObjectField {
    pub name: String,
    pub ty: DataType,
    pub optional: bool,
    pub flatten: bool,
}

#[derive(Debug, Clone)]
pub struct ObjectType {
    pub name: String,
    pub generics: Vec<&'static str>,
    pub fields: Vec<ObjectField>,
    pub tag: Option<String>,
    pub type_id: Option<TypeId>,
}

impl PartialEq for ObjectType {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}
