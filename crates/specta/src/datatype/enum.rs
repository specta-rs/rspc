use std::any::TypeId;

use crate::datatype::{DataType, ObjectType, TupleType};

/// this is used internally to represent the types.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct EnumType {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub generics: Vec<&'static str>,
    pub repr: EnumRepr,
    pub type_id: TypeId,
}

impl EnumType {
    /// An enum may contain variants which are invalid and will cause a runtime errors during serialize/deserialization.
    /// This function will filter them out so types can be exported for valid variants.
    pub fn make_flattenable(&mut self) {
        self.variants.retain(|v| match self.repr {
            EnumRepr::External => match v {
                EnumVariant::Unnamed(v) if v.fields.len() == 1 => true,
                EnumVariant::Named(_) => true,
                _ => false,
            },
            EnumRepr::Untagged => matches!(v, EnumVariant::Unit(_) | EnumVariant::Named(_)),
            EnumRepr::Adjacent { .. } => true,
            EnumRepr::Internal { .. } => {
                matches!(v, EnumVariant::Unit(_) | EnumVariant::Named(_))
            }
        });
    }
}

impl PartialEq for EnumType {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

/// this is used internally to represent the types.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum EnumRepr {
    External,
    Internal { tag: String },
    Adjacent { tag: String, content: String },
    Untagged,
}

/// this is used internally to represent the types.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum EnumVariant {
    Unit(String),
    Unnamed(TupleType),
    Named(ObjectType),
}

impl EnumVariant {
    /// Get the name of the variant.
    pub fn name(&self) -> &str {
        match self {
            Self::Unit(name) => name,
            Self::Unnamed(tuple_type) => &tuple_type.name,
            Self::Named(object_type) => &object_type.name,
        }
    }

    /// Get the [`DataType`](crate::DataType) of the variant.
    pub fn data_type(&self) -> DataType {
        match self {
            Self::Unit(_) => unreachable!("Unit enum variants have no type!"),
            Self::Unnamed(tuple_type) => tuple_type.clone().into(),
            Self::Named(object_type) => object_type.clone().into(),
        }
    }
}
