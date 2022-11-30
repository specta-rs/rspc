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
    /// An enum may contain variants which are invalid and will cause a runtime errors during serialize/deserialization. This function will filter them out so types can be exported for valid variants.
    #[allow(clippy::needless_collect)]
    pub fn make_flattenable(&mut self) {
        let indexes = self
            .variants
            .iter()
            .filter(|v| match self.repr {
                EnumRepr::External => match v {
                    EnumVariant::Unnamed(v) if v.fields.len() == 1 => false,
                    EnumVariant::Named(_) => false,
                    _ => true,
                },
                EnumRepr::Untagged => !matches!(v, EnumVariant::Unit(_) | EnumVariant::Named(_)),
                EnumRepr::Adjacent { .. } => false,
                EnumRepr::Internal { .. } => {
                    !matches!(v, EnumVariant::Unit(_) | EnumVariant::Named(_))
                }
            })
            .enumerate()
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        indexes.into_iter().rev().for_each(|i| {
            self.variants.remove(i);
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
            Self::Unnamed(tuple_type) => DataType::Tuple(tuple_type.clone()),
            Self::Named(object_type) => DataType::Object(object_type.clone()),
        }
    }
}
